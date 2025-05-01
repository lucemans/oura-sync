use std::sync::Arc;

use anyhow::Error;
use futures::join;
// use tracing::info;

pub mod database;
pub mod models;
pub mod modules;
pub mod server;
pub mod state;
// pub mod tmp;

#[async_std::main]
pub async fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let state = state::AppStateInner::init().await;
    let state = Arc::new(state);

    let oura_state = state.clone();
    let oura_handle = async_std::task::spawn(async move {
        oura_state.clone().oura.run(oura_state).await;
    });
    let oura2_state = state.clone();
    let server_handle = async_std::task::spawn(server::start_http(state.clone()));

    join!(server_handle, oura_handle);
    Ok(())
}
