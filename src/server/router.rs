use axum::{
    http::Method,
    routing::get,
    Json, Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use super::{
    handlers::{handle_manifest, handle_segment},
    state::AppState,
};

/// Create the application router.
pub async fn create_router() -> anyhow::Result<Router> {
    // Determine server base URL from environment or default
    let host = std::env::var("EXTERNAL_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let scheme = std::env::var("EXTERNAL_SCHEME").unwrap_or_else(|_| "http".to_string());
    
    let base_url = format!("{}://{}:{}", scheme, host, port);
    let server_base_url = url::Url::parse(&base_url)?;

    let state = AppState::new(server_base_url);

    // Configure CORS
    let cors_origin = std::env::var("CORS_ALLOWED_ORIGIN").unwrap_or_else(|_| "*".to_string());
    let cors = if cors_origin == "*" {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::OPTIONS])
            .allow_headers(Any)
    } else {
        CorsLayer::new()
            .allow_origin(cors_origin.parse::<axum::http::HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::OPTIONS])
            .allow_headers(Any)
    };

    let app = Router::new()
        .route("/manifest", get(handle_manifest))
        .route("/segment", get(handle_segment))
        .route("/segment.{ext}", get(handle_segment))
        .route("/health", get(health_check))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    Ok(app)
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}
