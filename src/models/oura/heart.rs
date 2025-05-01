use std::fmt::Display;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, FixedOffset};

use super::OuraMultiDocumentResponse;

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

pub type TimeSeriesResponseHeartRateModel = OuraMultiDocumentResponse<HeartRateModel>;
