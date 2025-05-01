use serde::{Serialize, Deserialize};
use chrono::NaiveDate;

/// Single cardiovascular age data point
#[derive(Debug, Serialize, Deserialize)]
pub struct CardiovascularAgeModel {
    /// Vascular age
    pub vascular_age: Option<i64>,

    /// Day of the measurement
    pub day: NaiveDate,
}
