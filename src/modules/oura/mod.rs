use crate::{Error, models::oura::OuraMultiDocumentResponse, state::AppState};
use async_std::sync::Mutex;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use futures::join;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

pub mod cva;
pub mod heart;
pub mod readiness;
pub mod sleep;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OuraConfig {
    pub token: String,
    pub start_date: Option<DateTime<Utc>>,
}

pub struct OuraService {
    pub config: OuraConfig,

    // Heart rate
    pub heart_cursor: Mutex<DateTime<Utc>>,
    pub heart_threshold_distance: Duration,

    // Cardiovascular age
    pub cva_cursor: Mutex<NaiveDate>,

    // Readiness
    pub readiness_cursor: Mutex<NaiveDate>,

    // Sleep
    pub sleep_cursor: Mutex<NaiveDate>,
}

pub const DEFAULT_START_DATE: DateTime<Utc> = DateTime::from_timestamp_micros(0).unwrap();

impl OuraService {
    pub async fn new(config: OuraConfig) -> Self {
        let start_date = config.start_date.unwrap_or(DEFAULT_START_DATE);

        let heart_cursor = Mutex::new(start_date);
        let cva_cursor = Mutex::new(start_date.date_naive());
        let readiness_cursor = Mutex::new(start_date.date_naive());
        let sleep_cursor = Mutex::new(start_date.date_naive());

        Self {
            config,
            heart_cursor,
            heart_threshold_distance: Duration::minutes(15),
            cva_cursor,
            readiness_cursor,
            sleep_cursor,
        }
    }

    pub async fn run(&self, state: AppState) {
        info!("Starting Oura service");

        join!(
            self.sync_cva(state.clone()),
            self.sync_heartrate(state.clone()),
            self.sync_readiness(state.clone()),
            self.sync_sleep(state.clone())
        );
    }

    pub async fn get_datetime_range(
        &self,
        url: &str,
        start_date: DateTime<Utc>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<OuraMultiDocumentResponse<serde_json::Value>, Error> {
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

        self.get_raw(url, params).await
    }

    pub async fn get_date_range(
        &self,
        url: &str,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Result<OuraMultiDocumentResponse<serde_json::Value>, Error> {
        let params = match end_date {
            Some(end_date) => {
                vec![
                    ("start_date", start_date.to_string()),
                    ("end_date", end_date.to_string()),
                ]
            }
            None => vec![("start_date", start_date.to_string())],
        };

        self.get_raw(url, params).await
    }

    pub async fn get_raw(
        &self,
        url: &str,
        params: Vec<(&str, String)>,
    ) -> Result<OuraMultiDocumentResponse<serde_json::Value>, Error> {
        // get the heartrate from the oura api
        let client = reqwest::Client::builder().use_rustls_tls().build().unwrap();

        let response = client
            .get(format!("https://api.ouraring.com/v2/{}", url))
            .bearer_auth(&self.config.token)
            .query(&params)
            .send()
            .await?;
        let body = response.text().await?;

        let response: OuraMultiDocumentResponse<serde_json::Value> =
            match serde_json::from_str(&body) {
                Ok(response) => response,
                Err(e) => {
                    error!("Error parsing {}: {} - {}", url, e, body);
                    return Err(e.into());
                }
            };

        info!("Found {} samples", response.data.len());

        Ok(response)
    }
}
