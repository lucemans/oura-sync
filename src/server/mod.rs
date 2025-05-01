use governor::Quota;
use poem::{
    EndpointExt, Route, Server,
    endpoint::StaticFilesEndpoint,
    get, handler,
    listener::TcpListener,
    middleware::{Cors, OpenTelemetryMetrics},
};
use poem_openapi::{OpenApi, OpenApiService, Tags, payload::Html};
use ratelimit::GovRateLimitMiddleware;
use user::UserApi;
use std::num::NonZero;
use tracing::info;

use crate::state::AppState;
pub mod user;

pub mod ratelimit;

#[derive(Tags)]
pub enum ApiTags {
    /// Topic Related Operations
    Topic,
    /// User Related Operations
    User,
    /// Events Related Operations
    Events,
}

fn get_api(state: AppState) -> impl OpenApi {
    (UserApi)
}

pub async fn start_http(state: AppState) {
    info!("Starting HTTP server");
    let api_service = OpenApiService::new(get_api(state.clone()), "Oura Sync", "0.0.1")
        .server("http://localhost:3000/api");

    let spec = api_service.spec_endpoint();

    let limiter = GovRateLimitMiddleware::new(
        Quota::per_minute(NonZero::new(120).unwrap()),
        Quota::per_minute(NonZero::new(60).unwrap()),
    );


    let api_service = api_service
        .with(limiter)
        // .with(TraceId::new(Arc::new(global::tracer("oura-sync"))))
        .with(OpenTelemetryMetrics::new());

    let path = std::path::Path::new("./www");

    let spa_endpoint = StaticFilesEndpoint::new(path)
        .show_files_listing()
        .index_file("index.html")
        .fallback_to_index();

    let app = Route::new()
        .nest("/", spa_endpoint)
        .nest("/openapi.json", spec)
        .nest("/docs", get(get_openapi_docs))
        .nest("/api", api_service)
        .data(state)
        .with(Cors::new());

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
        .unwrap();
}

#[handler]
async fn get_openapi_docs() -> Html<&'static str> {
    Html(include_str!("./index.html"))
}
