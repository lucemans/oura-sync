use std::fmt::Display;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, FixedOffset};

use super::OuraMultiDocumentResponse;

/// Single heart rate data point
#[derive(Debug, Serialize, Deserialize)]
pub struct CardiovascularAgeModel {
    /// Vascular age
    pub vascular_age: i64,

    /// Day of the measurement
    pub day: DateTime<FixedOffset>,
}

pub type TimeSeriesResponseCardiovascularAgeModel = OuraMultiDocumentResponse<CardiovascularAgeModel>;
