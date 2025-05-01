use crate::{Error, state::AppState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OuraConfig {
    pub token: String,
}

pub struct OuraService {
    pub config: OuraConfig,
}

impl OuraService {
    pub async fn new(config: OuraConfig) -> Self {
        Self { config }
    }

    pub async fn run(&self, state: AppState) -> Result<(), Error> {
        Ok(())
    }
}
