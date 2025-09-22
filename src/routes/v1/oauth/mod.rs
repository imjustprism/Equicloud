use axum::{
    routing::get,
    Router,
};

pub mod callback;
pub mod settings;

pub fn register() -> Router {
    Router::new()
        .route("/oauth/callback", get(callback::oauth_callback))
        .route("/oauth/settings", get(settings::oauth_settings))
}