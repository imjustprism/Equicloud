use axum::{
    Router,
    response::{IntoResponse, Json, Redirect, Response},
    routing::get,
};
use serde_json::{Value, json};
use std::env;
use tracing::debug;

pub fn register() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/", get(root_redirect))
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().timestamp(),
        "service": "equicloud"
    }))
}

async fn root_redirect() -> Response {
    if let Ok(redirect_url) = env::var("API_ROOT_REDIRECT_URL")
        && !redirect_url.is_empty()
    {
        debug!("Redirecting to: {}", redirect_url);
        if redirect_url.starts_with("http://") || redirect_url.starts_with("https://") {
            return Redirect::permanent(&redirect_url).into_response();
        } else {
            debug!("Invalid redirect URL format: {}", redirect_url);
        }
    }

    Json(json!({
        "message": "EquiCloud",
        "version": "1.0.0",
        "endpoints": [
            "/health",
            "/v1/oauth/callback",
            "/v1/oauth/settings",
            "/v1/settings"
        ]
    }))
    .into_response()
}
