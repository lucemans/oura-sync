use influxdb::Client;

use crate::state::DatabaseConfig;

pub struct Database {
    pub client: Client,
}

impl Database {
    pub async fn init(database_config: &DatabaseConfig) -> Self {
        let database = Self {
            client: Client::new(
                &database_config.url,
                database_config
                    .database_name
                    .clone()
                    .unwrap_or("oura-sync".to_string()),
            ),
        };

        database
    }
}
