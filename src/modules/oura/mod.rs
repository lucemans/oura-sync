use crate::{Error, models::oura::TimeSeriesResponseHeartRateModel, state::AppState};
use async_std::{sync::Mutex, task::sleep};
use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use futures::stream;
use influxdb2::models::DataPoint;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OuraConfig {
    pub token: String,
    pub start_date: Option<DateTime<Utc>>,
}

pub struct OuraService {
    pub config: OuraConfig,
    pub cursor: Mutex<DateTime<Utc>>,
    pub threshold_distance: Duration,
}

const DEFAULT_START_DATE: DateTime<Utc> = DateTime::from_timestamp_micros(0).unwrap();

impl OuraService {
    pub async fn new(config: OuraConfig) -> Self {
        let start_date = config.start_date.unwrap_or(DEFAULT_START_DATE);

        let cursor = Mutex::new(start_date);

        Self {
            config,
            cursor,
            threshold_distance: Duration::minutes(15),
        }
    }

    pub async fn run(&self, state: AppState) -> Result<(), Error> {
        info!("Starting Oura service");

        let resume_date = state.database.get_last_timestamp("heart").await.ok();
        if let Some(resume_date) = resume_date {
            info!("Resuming from {}", resume_date);
            *self.cursor.lock().await = resume_date;
        } else {
            info!("Starting from {}", self.cursor.lock().await);
        }

        loop {
            let now = Utc::now();
            let start_date = *self.cursor.lock().await;
            let end_date = if start_date == DEFAULT_START_DATE {
                Some(DEFAULT_START_DATE + Duration::days(30))
            } else {
                Some((start_date + chrono::Duration::days(1)).min(now))
            };

            if start_date > now - self.threshold_distance
                || end_date.unwrap_or(now) - start_date < self.threshold_distance
            {
                info!("Waiting for threshold distance to pass");
                sleep(self.threshold_distance.to_std().unwrap()).await;
                continue;
            }

            match self
                .index_heartrate(state.clone(), start_date, end_date)
                .await
            {
                Ok(_) => {
                    let cursor = *self.cursor.lock().await;
                    info!("Indexed heartrate from {} to {}", start_date, cursor);
                    // *self.cursor.lock().await = end_date;
                }
                Err(e) => {
                    error!("Error indexing heartrate: {}", e);
                }
            }
        }

        Ok(())
    }

    pub async fn index_heartrate(
        &self,
        state: AppState,
        start_date: DateTime<Utc>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<(), Error> {
        // TODO: start getting heartrate and add to influxdb
        let data = self
            .get_heartrate(state.clone(), start_date, end_date)
            .await?;

        info!("Next_token: {:?}", data.next_token);

        if data.data.is_empty() {
            info!(
                "No heartrate data found for range {} to {:?}",
                start_date.to_rfc3339(),
                end_date
            );
            if let Some(end_date) = end_date {
                *self.cursor.lock().await = end_date;
            }

            return Ok(());
        }

        let points: Vec<DataPoint> = data
            .data
            .iter()
            .map(|point| {
                DataPoint::builder("heart")
                    .tag("source", point.source.to_string())
                    .field("bpm", point.bpm)
                    .timestamp(point.timestamp.timestamp_nanos_opt().unwrap())
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
                error!("Error indexing heartrate: {}", e);
            }
        }

        let last_heartrate_timestamp = data.data.last().unwrap().timestamp;
        info!(
            "Last heartrate timestamp: {}, next token: {:?}",
            last_heartrate_timestamp, data.next_token
        );

        *self.cursor.lock().await = last_heartrate_timestamp.to_utc() + Duration::seconds(1);

        Ok(())
    }

    // time range may not exceed 30 days
    pub async fn get_heartrate(
        &self,
        state: AppState,
        start_date: DateTime<Utc>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<TimeSeriesResponseHeartRateModel, Error> {
        // get the heartrate from the oura api
        let client = reqwest::Client::builder().use_rustls_tls().build().unwrap();

        let params = match end_date {
            Some(end_date) => {
                info!("Fetching with end time: {}", end_date.to_rfc3339());
                vec![
                    ("start_datetime", start_date.to_rfc3339()),
                    ("end_datetime", end_date.to_rfc3339()),
                ]
            }
            None => {
                info!("Fetching without end time");
                vec![
                    ("start_datetime", start_date.to_rfc3339()),
                    ("end_datetime", "null".to_string()),
                ]
            }
        };

        let response = client
            .get("https://api.ouraring.com/v2/usercollection/heartrate")
            .bearer_auth(&self.config.token)
            .query(&params)
            .send()
            .await?;
        let body = response.text().await?;

        let heartrate: TimeSeriesResponseHeartRateModel = match serde_json::from_str(&body) {
            Ok(heartrate) => heartrate,
            Err(e) => {
                error!("Error parsing heartrate: {} - {}", e, body);
                return Err(e.into());
            }
        };

        info!("Found {} heartrate samples", heartrate.data.len());

        Ok(heartrate)
    }
}
