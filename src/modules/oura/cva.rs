use anyhow::Error;
use async_std::task::sleep;
use chrono::{Duration, NaiveDate, Utc};
use futures::stream;
use influxdb2::models::DataPoint;
use tracing::{error, info};

use crate::{
    models::oura::{cva::CardiovascularAgeModel, OuraMultiDocumentResponse},
    state::AppState,
};

use super::{DEFAULT_START_DATE, OuraService};

pub const INFLUX_MEASUREMENT_CVA: &str = "cardiovascular_age";

impl OuraService {
    pub async fn sync_cva(&self, state: AppState) {
        let resume_date = state.database.get_last_timestamp(INFLUX_MEASUREMENT_CVA).await.ok();
        if let Some(resume_date) = resume_date {
            info!("Resuming from {}", resume_date);
            *self.cva_cursor.lock().await = resume_date.date_naive();
        } else {
            info!("Starting from {}", self.cva_cursor.lock().await);
        }

        loop {
            let now = Utc::now();
            let start_date = *self.cva_cursor.lock().await;
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
                .index_cva(state.clone(), start_date, end_date)
                .await
            {
                Ok(_) => {
                    let cursor = *self.cva_cursor.lock().await;
                    info!("Indexed cardiovascular age from {} to {}", start_date, cursor);
                    // *self.cursor.lock().await = end_date;
                }
                Err(e) => {
                    error!("Error indexing cardiovascular age: {}", e);
                }
            }
        }
    }

    pub async fn index_cva(
        &self,
        state: AppState,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Result<(), Error> {
        // TODO: start getting heartrate and add to influxdb
        let data = self.get_cva(start_date, end_date).await?;

        info!("Next_token: {:?}", data.next_token);

        if data.data.is_empty() {
            info!(
                "No cardiovascular age data found for range {} to {:?}",
                start_date.to_string(),
                end_date
            );
            if let Some(end_date) = end_date {
                *self.cva_cursor.lock().await = end_date;
            }

            return Ok(());
        }

        let points: Vec<DataPoint> = data
            .data
            .iter()
            .map(|point| {
                DataPoint::builder(INFLUX_MEASUREMENT_CVA)
                    .field("vascular_age", point.vascular_age.unwrap_or(0))
                    .timestamp(point.day.and_hms_opt(0, 0, 0).unwrap().timestamp_nanos_opt().unwrap())
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
                error!("Error indexing cardiovascular age: {}", e);
            }
        }

        let last_cva_timestamp = data.data.last().unwrap().day;
        info!(
            "Last cardiovascular age timestamp: {}, next token: {:?}",
            last_cva_timestamp, data.next_token
        );

        *self.cva_cursor.lock().await = last_cva_timestamp + Duration::days(1);

        Ok(())
    }

    // time range may not exceed 30 days
    pub async fn get_cva(
        &self,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Result<OuraMultiDocumentResponse<CardiovascularAgeModel>, Error> {
        let x = self
            .get_date_range("usercollection/daily_cardiovascular_age", start_date, end_date)
            .await?;

        let next_token = x.next_token;
        let x: Vec<CardiovascularAgeModel> = x
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
