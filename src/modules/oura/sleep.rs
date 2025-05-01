use anyhow::Error;
use async_std::task::sleep;
use chrono::{Duration, NaiveDate, Utc};
use futures::stream;
use influxdb2::models::DataPoint;
use tracing::{error, info};

use crate::{
    models::oura::{OuraMultiDocumentResponse, sleep::SleepModel},
    state::AppState,
};

use super::{DEFAULT_START_DATE, OuraService};

pub const INFLUX_MEASUREMENT_SLEEP: &str = "sleep";

impl OuraService {
    pub async fn sync_sleep(&self, state: AppState) {
        let resume_date = state
            .database
            .get_last_timestamp(INFLUX_MEASUREMENT_SLEEP)
            .await
            .ok();
        if let Some(resume_date) = resume_date {
            info!("Resuming from {}", resume_date);
            *self.sleep_cursor.lock().await = resume_date.date_naive();
        } else {
            info!("Starting from {}", self.sleep_cursor.lock().await);
        }

        loop {
            let now = Utc::now();
            let start_date = *self.sleep_cursor.lock().await;
            let end_date = if start_date == DEFAULT_START_DATE.date_naive() {
                Some(DEFAULT_START_DATE.date_naive() + Duration::days(30))
            } else {
                Some((start_date + chrono::Duration::days(10)).min(now.date_naive()))
            };

            if start_date > now.date_naive() - Duration::days(1)
                || end_date.unwrap_or(now.date_naive()) - start_date < Duration::days(1)
            {
                info!("Waiting for threshold distance to pass");
                sleep(Duration::days(1).to_std().unwrap()).await;
                continue;
            }

            match self.index_sleep(state.clone(), start_date, end_date).await {
                Ok(_) => {
                    let cursor = *self.sleep_cursor.lock().await;
                    info!("Indexed sleep from {} to {}", start_date, cursor);
                    // *self.cursor.lock().await = end_date;
                }
                Err(e) => {
                    error!("Error indexing sleep: {}", e);
                }
            }
        }
    }

    pub async fn index_sleep(
        &self,
        state: AppState,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Result<(), Error> {
        // TODO: start getting heartrate and add to influxdb
        let data = self.get_sleep(start_date, end_date).await?;

        info!("Next_token: {:?}", data.next_token);

        if data.data.is_empty() {
            info!(
                "No sleep data found for range {} to {:?}",
                start_date.to_string(),
                end_date
            );
            if let Some(end_date) = end_date {
                *self.sleep_cursor.lock().await = end_date;
            }

            return Ok(());
        }

        let points: Vec<DataPoint> = data
            .data
            .iter()
            .filter_map(|point| {
                let mut builder = DataPoint::builder(INFLUX_MEASUREMENT_SLEEP);

                if let Some(average_breath) = point.average_breath {
                    builder = builder.field("average_breath", average_breath);
                }
                if let Some(average_heart_rate) = point.average_heart_rate {
                    builder = builder.field("average_heart_rate", average_heart_rate);
                }
                if let Some(average_hrv) = point.average_hrv {
                    builder = builder.field("average_hrv", average_hrv);
                }
                if let Some(awake_time) = point.awake_time {
                    builder = builder.field("awake_time", awake_time);
                }
                if let Some(bedtime_end) = point.bedtime_end {
                    builder = builder.field("bedtime_end", bedtime_end.to_string());
                }
                if let Some(bedtime_start) = point.bedtime_start {
                    builder = builder.field("bedtime_start", bedtime_start.to_string());
                }
                if let Some(deep_sleep_duration) = point.deep_sleep_duration {
                    builder = builder.field("deep_sleep_duration", deep_sleep_duration);
                }
                if let Some(efficiency) = point.efficiency {
                    builder = builder.field("efficiency", efficiency);
                }
                if let Some(latency) = point.latency {
                    builder = builder.field("latency", latency);
                }
                if let Some(light_sleep_duration) = point.light_sleep_duration {
                    builder = builder.field("light_sleep_duration", light_sleep_duration);
                }
                if let Some(low_battery_alert) = point.low_battery_alert {
                    builder = builder.field("low_battery_alert", low_battery_alert);
                }
                if let Some(movement_30_sec) = &point.movement_30_sec {
                    builder = builder.field("movement_30_sec", movement_30_sec.as_str());
                }
                if let Some(period) = point.period {
                    builder = builder.field("period", period);
                }
                if let Some(readiness_score_delta) = point.readiness_score_delta {
                    builder = builder.field("readiness_score_delta", readiness_score_delta);
                }
                if let Some(rem_sleep_duration) = point.rem_sleep_duration {
                    builder = builder.field("rem_sleep_duration", rem_sleep_duration);
                }
                if let Some(restless_periods) = point.restless_periods {
                    builder = builder.field("restless_periods", restless_periods);
                }
                if let Some(sleep_phase_5_min) = &point.sleep_phase_5_min {
                    builder = builder.field("sleep_phase_5_min", sleep_phase_5_min.as_str());
                }
                if let Some(sleep_score_delta) = point.sleep_score_delta {
                    builder = builder.field("sleep_score_delta", sleep_score_delta);
                }
                if let Some(sleep_algorithm_version) = &point.sleep_algorithm_version {
                    builder = builder.field(
                        "sleep_algorithm_version",
                        sleep_algorithm_version.as_str(),
                    );
                }
                if let Some(time_in_bed) = point.time_in_bed {
                    builder = builder.field("time_in_bed", time_in_bed);
                }
                if let Some(total_sleep_duration) = point.total_sleep_duration {
                    builder = builder.field("total_sleep_duration", total_sleep_duration);
                }
                if let Some(_type) = &point._type {
                    builder = builder.field("type", _type.as_str());
                }
                if let Some(id) = &point.id {
                    builder = builder.tag("id", id);
                }

                builder = builder
                    .timestamp(
                        point
                            .bedtime_start
                            .unwrap_or_else(|| {
                                point
                                    .day
                                    .and_hms_opt(0, 0, 0)
                                    .unwrap()
                                    .and_utc()
                                    .fixed_offset()
                            })
                            .timestamp_nanos_opt()
                            .unwrap(),
                    );

                match builder.build() {
                    Ok(dp) => Some(dp),
                    Err(e) => {
                        error!("Error building data point: {} - {:?}", e, point);
                        None
                    }
                }
            })
            .collect();

        match state
            .database
            .client
            .write(&state.database.bucket, stream::iter(points))
            .await
        {
            Ok(_) => {
                info!("Indexed");
            }
            Err(e) => {
                error!("Error indexing sleep: {}", e);
            }
        }

        let last_sleep_timestamp = data.data.last().unwrap().day;
        info!(
            "Last sleep timestamp: {}, next token: {:?}",
            last_sleep_timestamp, data.next_token
        );

        *self.sleep_cursor.lock().await = last_sleep_timestamp + Duration::days(1);

        Ok(())
    }

    // time range may not exceed 30 days
    pub async fn get_sleep(
        &self,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Result<OuraMultiDocumentResponse<SleepModel>, Error> {
        let x = self
            .get_date_range("usercollection/sleep", start_date, end_date)
            .await?;

        let next_token = x.next_token;
        let x: Vec<SleepModel> = x
            .data
            .into_iter()
            .map(|x| serde_json::from_value(x).unwrap())
            .collect();

        Ok(OuraMultiDocumentResponse {
            data: x,
            next_token,
        })
    }
}
