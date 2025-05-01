use serde::{Deserialize, Serialize};

pub mod heart;
pub mod cva;

/// Time series response for heart rate endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct OuraMultiDocumentResponse<T> {
    /// Array of heart rate data points
    pub data: Vec<T>,

    /// Pagination token for next page
    #[serde(rename = "next_token")]
    pub next_token: Option<String>,
}

