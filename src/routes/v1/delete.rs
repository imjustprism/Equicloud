use axum::{
    Extension,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;
use tracing::error;

use equicloud::DatabaseService;

pub async fn get_user_info() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().timestamp(),
        "service": "equicloud"
    }))
}

pub async fn delete_all_user_data(
    Extension(db): Extension<DatabaseService>,
    Extension(user_id): Extension<String>,
) -> impl IntoResponse {
    match db.delete_user_settings(&user_id).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(e) => {
            error!("Database error in delete_all_user_data: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
