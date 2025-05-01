use std::fmt::Display;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, FixedOffset};

/// Query parameters for fetching heart rate data
#[derive(Debug, Serialize, Deserialize)]
pub struct HeartrateQueryParams {
    /// Start datetime (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none", rename = "start_datetime")]
    pub start_datetime: Option<DateTime<FixedOffset>>,

    /// End datetime (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none", rename = "end_datetime")]
    pub end_datetime: Option<DateTime<FixedOffset>>,

    /// Pagination token
    #[serde(skip_serializing_if = "Option::is_none", rename = "next_token")]
    pub next_token: Option<String>,
}

/// Source of the heart rate measurement
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HeartRateSource {
    Awake,
    Rest,
    Sleep,
    Session,
    Live,
    Workout,
}

impl Display for HeartRateSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).unwrap().trim_matches('"').to_string();
        write!(f, "{}", s)
    }
}

/// Single heart rate data point
#[derive(Debug, Serialize, Deserialize)]
pub struct HeartRateModel {
    /// Beats per minute
    pub bpm: i64,

    /// Source of the measurement
    pub source: HeartRateSource,

    /// Timestamp of the measurement (ISO 8601)
    pub timestamp: DateTime<FixedOffset>,
}

/// Time series response for heart rate endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSeriesResponseHeartRateModel {
    /// Array of heart rate data points
    pub data: Vec<HeartRateModel>,

    /// Pagination token for next page
    #[serde(rename = "next_token")]
    pub next_token: Option<String>,
}
