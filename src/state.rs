use crate::{database::Database, modules::oura::{OuraConfig, OuraService}};
use figment::{providers::Env, Figment};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub type AppState = Arc<AppStateInner>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub database_name: Option<String>,
}

pub struct AppStateInner {
    pub database: Database,
    pub oura: OuraService,
}

impl AppStateInner {
    pub async fn init() -> Self {
        // Load configuration from environment variables
        let database_config = Figment::new()
            .merge(Env::prefixed("DATABASE_"))
            .extract::<DatabaseConfig>()
            .expect("Failed to load database configuration");

        let database = Database::init(&database_config).await;

        let oura_config = Figment::new()
            .merge(Env::prefixed("OURA_"))
            .extract::<OuraConfig>()
            .expect("Failed to load oura configuration");

        let oura = OuraService::new(oura_config).await;

        Self {
            database,
            oura,
        }
    }
}

impl std::fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppStateInner")
            // .field("database", &self.database)
            .finish()
    }
}
