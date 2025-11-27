use axum::{
    Extension,
    body::{Body, Bytes},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde_json::json;
use tracing::error;

use equicloud::DatabaseService;
use equicloud::utils::{CONFIG, error_response};

pub async fn head_settings(
    Extension(db): Extension<DatabaseService>,
    Extension(user_id): Extension<String>,
    _headers: HeaderMap,
) -> impl IntoResponse {
    match db.get_settings_metadata(&user_id).await {
        Ok(Some(written)) => {
            let mut response_headers = HeaderMap::new();
            if let Ok(etag_value) = written.parse() {
                response_headers.insert("ETag", etag_value);
            } else {
                error!("Failed to parse ETag value: {}", written);
            }
            (StatusCode::NO_CONTENT, response_headers)
        }
        Ok(None) => (StatusCode::NOT_FOUND, HeaderMap::new()),
        Err(e) => {
            error!("Database error in head_settings: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new())
        }
    }
}

pub async fn get_settings(
    Extension(db): Extension<DatabaseService>,
    Extension(user_id): Extension<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    match db.get_user_settings(&user_id).await {
        Ok(Some((value, written))) => {
            if let Some(if_none_match) = headers.get("if-none-match")
                && if_none_match.to_str().unwrap_or("") == written
            {
                return (StatusCode::NOT_MODIFIED, HeaderMap::new(), Body::empty()).into_response();
            }

            let mut response_headers = HeaderMap::new();
            if let Ok(content_type) = "application/octet-stream".parse() {
                response_headers.insert("Content-Type", content_type);
            }
            if let Ok(etag_value) = written.parse() {
                response_headers.insert("ETag", etag_value);
            } else {
                error!("Failed to parse ETag value: {}", written);
            }

            (StatusCode::OK, response_headers, Body::from(value)).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, HeaderMap::new(), Body::empty()).into_response(),
        Err(e) => {
            error!("Database error in get_settings: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(error_response("Failed to retrieve settings")),
            )
                .into_response()
        }
    }
}

pub async fn put_settings(
    Extension(db): Extension<DatabaseService>,
    Extension(user_id): Extension<String>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    if headers.get("content-type").and_then(|h| h.to_str().ok()) != Some("application/octet-stream")
    {
        return (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            axum::Json(error_response(
                "Content type must be application/octet-stream",
            )),
        )
            .into_response();
    }

    let size_limit = CONFIG.max_backup_size_bytes;

    if body.len() > size_limit {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            axum::Json(error_response("Settings are too large")),
        )
            .into_response();
    }

    match db.save_user_settings(&user_id, body.to_vec()).await {
        Ok(written) => (
            StatusCode::OK,
            axum::Json(json!({
                "written": written
            })),
        )
            .into_response(),
        Err(e) => {
            error!("Database error in put_settings: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(error_response("Failed to save settings")),
            )
                .into_response()
        }
    }
}

pub async fn delete_settings(
    Extension(db): Extension<DatabaseService>,
    Extension(user_id): Extension<String>,
) -> impl IntoResponse {
    match db.delete_user_settings(&user_id).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(e) => {
            error!("Database error in delete_settings: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
