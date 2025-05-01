use poem::web::Data;
use poem::Result;
use poem_openapi::payload::Json;
use poem_openapi::{Object, OpenApi};
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use crate::server::ApiTags;

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct UserApi;

#[OpenApi]
impl UserApi {
    /// /users
    ///
    /// List users
    #[oai(path = "/users", method = "get", tag = "ApiTags::User")]
    async fn list(
        &self,
        state: Data<&AppState>,
    ) -> Result<Json<serde_json::Value>> {
        Ok(Json(serde_json::Value::Null))
    }
}
