use std::collections::HashMap;

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};

/// Single cardiovascular age data point
#[derive(Debug, Serialize, Deserialize)]
pub struct SleepModel {
    /// Readiness score
    pub id: Option<String>,

    pub average_breath: Option<f64>,
    pub average_heart_rate: Option<f64>,
    pub average_hrv: Option<f64>,
    pub awake_time: Option<i64>,
    pub bedtime_end: Option<DateTime<FixedOffset>>,
    pub bedtime_start: Option<DateTime<FixedOffset>>,
    pub deep_sleep_duration: Option<i64>,
    pub efficiency: Option<i64>,
    // pub heart_rate: SleepHeartRateModel,
    // pub hrv:
    pub latency: Option<i64>,
    pub light_sleep_duration: Option<i64>,
    pub low_battery_alert: Option<bool>,
    pub movement_30_sec: Option<String>, // not a clue
    pub period: Option<i64>,             // no clue
    // pub readiness
    pub readiness_score_delta: Option<i64>,
    pub rem_sleep_duration: Option<i64>,
    pub restless_periods: Option<i64>,
    pub sleep_phase_5_min: Option<String>,
    pub sleep_score_delta: Option<i64>,
    pub sleep_algorithm_version: Option<String>,
    pub time_in_bed: Option<i64>,
    pub total_sleep_duration: Option<i64>,
    #[serde(rename = "type")]
    pub _type: Option<String>,

    /// Day of the measurement
    pub day: NaiveDate,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
