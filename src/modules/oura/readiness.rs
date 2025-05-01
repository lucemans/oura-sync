use anyhow::Error;
use async_std::task::sleep;
use chrono::{Duration, NaiveDate, Utc};
use futures::stream;
use influxdb2::models::DataPoint;
use tracing::{error, info};

use crate::{
    models::oura::{OuraMultiDocumentResponse, readiness::ReadinessModel},
    state::AppState,
};

use super::{DEFAULT_START_DATE, OuraService};

pub const INFLUX_MEASUREMENT_READINESS: &str = "readiness";

impl OuraService {
    pub async fn sync_readiness(&self, state: AppState) {
        let resume_date = state
            .database
            .get_last_timestamp(INFLUX_MEASUREMENT_READINESS)
            .await
            .ok();
        if let Some(resume_date) = resume_date {
            info!("Resuming from {}", resume_date);
            *self.readiness_cursor.lock().await = resume_date.date_naive();
        } else {
            info!("Starting from {}", self.readiness_cursor.lock().await);
        }

        loop {
            let now = Utc::now();
            let start_date = *self.readiness_cursor.lock().await;
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

            match self
                .index_readiness(state.clone(), start_date, end_date)
                .await
            {
                Ok(_) => {
                    let cursor = *self.readiness_cursor.lock().await;
                    info!("Indexed readiness from {} to {}", start_date, cursor);
                    // *self.cursor.lock().await = end_date;
                }
                Err(e) => {
                    error!("Error indexing readiness: {}", e);
                }
            }
        }
    }

    pub async fn index_readiness(
        &self,
        state: AppState,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Result<(), Error> {
        // TODO: start getting heartrate and add to influxdb
        let data = self.get_readiness(start_date, end_date).await?;

        info!("Next_token: {:?}", data.next_token);

        if data.data.is_empty() {
            info!(
                "No readiness data found for range {} to {:?}",
                start_date.to_string(),
                end_date
            );
            if let Some(end_date) = end_date {
                *self.readiness_cursor.lock().await = end_date;
            }

            return Ok(());
        }

        let points: Vec<DataPoint> = data
            .data
            .iter()
            .map(|point| {
                let mut builder = DataPoint::builder(INFLUX_MEASUREMENT_READINESS);

                if let Some(score) = point.score {
                    builder = builder.field("score", score);
                }

                if let Some(contributors) = &point.contributors {
                    if let Some(activity_balance) = contributors.activity_balance {
                        builder = builder.field("activity_balance", activity_balance);
                    }
                    if let Some(body_temperature) = contributors.body_temperature {
                        builder = builder.field("body_temperature", body_temperature);
                    }
                    if let Some(hrv_balance) = contributors.hrv_balance {
                        builder = builder.field("hrv_balance", hrv_balance);
                    }
                    if let Some(previous_day_activity) = contributors.previous_day_activity {
                        builder = builder.field("previous_day_activity", previous_day_activity);
                    }
                    if let Some(previous_night) = contributors.previous_night {
                        builder = builder.field("previous_night", previous_night);
                    }
                    if let Some(recovery_index) = contributors.recovery_index {
                        builder = builder.field("recovery_index", recovery_index);
                    }
                    if let Some(resting_heart_rate) = contributors.resting_heart_rate {
                        builder = builder.field("resting_heart_rate", resting_heart_rate);
                    }
                    if let Some(sleep_balance) = contributors.sleep_balance {
                        builder = builder.field("sleep_balance", sleep_balance);
                    }
                }

                if let Some(temperature_deviation) = point.temperature_deviation {
                    builder = builder.field("temperature_deviation", temperature_deviation);
                }
                if let Some(temperature_trend_deviation) = point.temperature_trend_deviation {
                    builder =
                        builder.field("temperature_trend_deviation", temperature_trend_deviation);
                }

                if let Some(id) = &point.id {
                    builder = builder.tag("id", id);
                }

                builder
                    .timestamp(
                        point
                            .day
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                            .timestamp_nanos_opt()
                            .unwrap(),
                    )
                    .build()
                    .unwrap()
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
                error!("Error indexing readiness: {}", e);
            }
        }

        let last_readiness_timestamp = data.data.last().unwrap().day;
        info!(
            "Last readiness timestamp: {}, next token: {:?}",
            last_readiness_timestamp, data.next_token
        );

        *self.readiness_cursor.lock().await = last_readiness_timestamp + Duration::days(1);

        Ok(())
    }

    // time range may not exceed 30 days
    pub async fn get_readiness(
        &self,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Result<OuraMultiDocumentResponse<ReadinessModel>, Error> {
        let x = self
            .get_date_range("usercollection/daily_readiness", start_date, end_date)
            .await?;

        let next_token = x.next_token;
        let x: Vec<ReadinessModel> = x
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
