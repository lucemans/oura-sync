use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use influxdb2::{models::Query, Client, FromDataPoint};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub org: String,
    pub token: String,
    pub bucket: Option<String>,
}

pub struct Database {
    pub client: Client,
    pub bucket: String,
}

#[derive(Debug, FromDataPoint, Default)]
pub struct HeartRatePoint {
    // pub source: String,
    // pub bpm: i64,
    pub time: DateTime<FixedOffset>,
}

impl Database {
    pub async fn init(database_config: &DatabaseConfig) -> Self {
        let database = Self {
            client: Client::new(
                &database_config.url,
                &database_config.org,
                &database_config.token,
            ),
            bucket: database_config.bucket.clone().unwrap_or("oura-sync".to_string()),
        };

        database
    }

    pub async fn get_last_timestamp(&self, measurement: &str) -> Result<DateTime<Utc>, anyhow::Error> {
        // InfluxDB 2.x uses Flux, not InfluxQL. Let's use a Flux query to get the latest timestamp.
        let qs = format!(
            r#"
            from(bucket: "{}")
                |> range(start: 0)
                |> filter(fn: (r) => r["_measurement"] == "{}")
                |> sort(columns: ["_time"], desc: true)
                |> limit(n:1)
            "#,
            self.bucket, measurement
        );
        let flux_query = Query::new(qs);

        let mut query_result: Vec<HeartRatePoint> = self.client.query(Some(flux_query)).await?;

        let last_timestamp = query_result.first().ok_or_else(|| anyhow::anyhow!("No records found for measurement '{}'", measurement))?.time;

        Ok(last_timestamp.to_utc())
    }
}
