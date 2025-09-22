use axum::Router;

pub mod health;
pub mod v1;

pub fn register_routes() -> Router {
    Router::new()
        .merge(health::register())
        .nest("/v1", v1::register())
}
