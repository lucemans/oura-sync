use anyhow::Error;
use async_std::task::sleep;
use chrono::{DateTime, Duration, Utc};
use futures::stream;
use influxdb2::models::DataPoint;
use tracing::{error, info};

use crate::{
    models::oura::{OuraMultiDocumentResponse, heart::HeartRateModel},
    state::AppState,
};

use super::{DEFAULT_START_DATE, OuraService};

impl OuraService {
    pub async fn sync_heartrate(&self, state: AppState) {
        let resume_date = state.database.get_last_timestamp("heart").await.ok();
        if let Some(resume_date) = resume_date {
            info!("Resuming from {}", resume_date);
            *self.heart_cursor.lock().await = resume_date;
        } else {
            info!("Starting from {}", self.heart_cursor.lock().await);
        }

        loop {
            let now = Utc::now();
            let start_date = *self.heart_cursor.lock().await;
            let end_date = if start_date == DEFAULT_START_DATE {
                Some(DEFAULT_START_DATE + Duration::days(30))
            } else {
                Some((start_date + chrono::Duration::days(1)).min(now))
            };

            if start_date > now - self.heart_threshold_distance
                || end_date.unwrap_or(now) - start_date < self.heart_threshold_distance
            {
                info!("Waiting for threshold distance to pass");
                sleep(self.heart_threshold_distance.to_std().unwrap()).await;
                continue;
            }

            match self
                .index_heartrate(state.clone(), start_date, end_date)
                .await
            {
                Ok(_) => {
                    let cursor = *self.heart_cursor.lock().await;
                    info!("Indexed heartrate from {} to {}", start_date, cursor);
                    // *self.cursor.lock().await = end_date;
                }
                Err(e) => {
                    error!("Error indexing heartrate: {}", e);
                }
            }
        }
    }

    pub async fn index_heartrate(
        &self,
        state: AppState,
        start_date: DateTime<Utc>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<(), Error> {
        // TODO: start getting heartrate and add to influxdb
        let data = self.get_heartrate(start_date, end_date).await?;

        info!("Next_token: {:?}", data.next_token);

        if data.data.is_empty() {
            info!(
                "No heartrate data found for range {} to {:?}",
                start_date.to_rfc3339(),
                end_date
            );
            if let Some(end_date) = end_date {
                *self.heart_cursor.lock().await = end_date;
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

        *self.heart_cursor.lock().await = last_heartrate_timestamp.to_utc() + Duration::seconds(1);

        Ok(())
    }

    // time range may not exceed 30 days
    pub async fn get_heartrate(
        &self,
        start_date: DateTime<Utc>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<OuraMultiDocumentResponse<HeartRateModel>, Error> {
        let x = self
            .get_datetime_range("usercollection/heartrate", start_date, end_date)
            .await?;

        let next_token = x.next_token;
        let x: Vec<HeartRateModel> = x
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
