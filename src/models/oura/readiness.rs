use serde::{Serialize, Deserialize};
use chrono::NaiveDate;

/// Single cardiovascular age data point
#[derive(Debug, Serialize, Deserialize)]
pub struct ReadinessModel {
    /// Readiness score
    pub id: Option<String>,

    pub contributors: Option<ReadinessContributors>,

    pub score: Option<i64>,
    pub temperature_deviation: Option<f64>,
    pub temperature_trend_deviation: Option<f64>,
    pub timestamp: Option<String>,

    /// Day of the measurement
    pub day: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadinessContributors {
    pub activity_balance: Option<i64>,
    pub body_temperature: Option<i64>,
    pub hrv_balance: Option<i64>,
    pub previous_day_activity: Option<i64>,
    pub previous_night: Option<i64>,
    pub recovery_index: Option<i64>,
    pub resting_heart_rate: Option<i64>,
    pub sleep_balance: Option<i64>,
}
