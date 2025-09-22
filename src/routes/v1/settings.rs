use anyhow::Result;
use axum::{
    body::Body,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use chrono::Utc;
use scylla::client::session_builder::SessionBuilder;
use serde_json::json;
use std::env;

pub async fn head_settings(_headers: HeaderMap, user_id: String) -> impl IntoResponse {
    match get_settings_metadata(&user_id).await {
        Ok(Some(written)) => {
            let mut response_headers = HeaderMap::new();
            response_headers.insert("ETag", written.parse().unwrap());
            (StatusCode::NO_CONTENT, response_headers)
        }
        Ok(None) => (StatusCode::NOT_FOUND, HeaderMap::new()),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, HeaderMap::new()),
    }
}

pub async fn get_settings(headers: HeaderMap, user_id: String) -> impl IntoResponse {
    match get_user_settings(&user_id).await {
        Ok(Some((value, written))) => {
            if let Some(if_none_match) = headers.get("if-none-match") {
                if if_none_match.to_str().unwrap_or("") == written {
                    return (StatusCode::NOT_MODIFIED, HeaderMap::new(), Body::empty())
                        .into_response();
                }
            }

            let mut response_headers = HeaderMap::new();
            response_headers.insert("Content-Type", "application/octet-stream".parse().unwrap());
            response_headers.insert("ETag", written.parse().unwrap());

            (StatusCode::OK, response_headers, Body::from(value)).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, HeaderMap::new(), Body::empty()).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            HeaderMap::new(),
            Body::empty(),
        )
            .into_response(),
    }
}

pub async fn put_settings(headers: HeaderMap, user_id: String, body: Vec<u8>) -> impl IntoResponse {
    if headers.get("content-type").and_then(|h| h.to_str().ok()) != Some("application/octet-stream")
    {
        return (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            axum::Json(json!({
                "error": "Content type must be application/octet-stream"
            })),
        )
            .into_response();
    }

    let size_limit = env::var("MAX_BACKUP_SIZE_BYTES")
        .unwrap_or_else(|_| "62914560".to_string())
        .parse::<usize>()
        .unwrap_or(62914560);

    if body.len() > size_limit {
        return (
            StatusCode::PAYLOAD_TOO_LARGE,
            axum::Json(json!({
                "error": "Settings are too large"
            })),
        )
            .into_response();
    }

    match save_user_settings(&user_id, body).await {
        Ok(written) => (
            StatusCode::OK,
            axum::Json(json!({
                "written": written
            })),
        )
            .into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({
                "error": "Failed to save settings"
            })),
        )
            .into_response(),
    }
}

pub async fn delete_settings(user_id: String) -> impl IntoResponse {
    match delete_user_settings(&user_id).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn get_settings_metadata(user_id: &str) -> Result<Option<String>> {
    let scylla_uri = env::var("SCYLLA_URI").unwrap_or_default();
    let username = env::var("SCYLLA_USERNAME").ok();
    let password = env::var("SCYLLA_PASSWORD").ok();

    let mut session_builder = SessionBuilder::new().known_node(&scylla_uri);

    if let (Some(user), Some(pass)) = (username, password) {
        session_builder = session_builder.user(&user, &pass);
    }

    let session = session_builder.build().await?;
    session.use_keyspace("equicloud", false).await?;

    let user_hash = crc32fast::hash(user_id.as_bytes());
    let hash_key = format!("settings:{}", user_hash);

    let query = "SELECT updated_at FROM users WHERE id = ?";
    let result = session.query_unpaged(query, (&hash_key,)).await?;

    let rows_result = result.into_rows_result()?;
    for row in rows_result.rows::<(i64,)>()? {
        let (updated_at,) = row?;
        return Ok(Some(updated_at.to_string()));
    }

    Ok(None)
}

async fn get_user_settings(user_id: &str) -> Result<Option<(Vec<u8>, String)>> {
    let scylla_uri = env::var("SCYLLA_URI").unwrap_or_default();
    let username = env::var("SCYLLA_USERNAME").ok();
    let password = env::var("SCYLLA_PASSWORD").ok();

    let mut session_builder = SessionBuilder::new().known_node(&scylla_uri);

    if let (Some(user), Some(pass)) = (username, password) {
        session_builder = session_builder.user(&user, &pass);
    }

    let session = session_builder.build().await?;
    session.use_keyspace("equicloud", false).await?;

    let user_hash = crc32fast::hash(user_id.as_bytes());
    let hash_key = format!("settings:{}", user_hash);

    let query = "SELECT settings, updated_at FROM users WHERE id = ?";
    let result = session.query_unpaged(query, (&hash_key,)).await?;

    let rows_result = result.into_rows_result()?;
    for row in rows_result.rows::<(Vec<u8>, i64)>()? {
        let (settings, updated_at) = row?;
        return Ok(Some((settings, updated_at.to_string())));
    }

    Ok(None)
}

async fn save_user_settings(user_id: &str, settings: Vec<u8>) -> Result<i64> {
    let scylla_uri = env::var("SCYLLA_URI").unwrap_or_default();
    let username = env::var("SCYLLA_USERNAME").ok();
    let password = env::var("SCYLLA_PASSWORD").ok();

    let mut session_builder = SessionBuilder::new().known_node(&scylla_uri);

    if let (Some(user), Some(pass)) = (username, password) {
        session_builder = session_builder.user(&user, &pass);
    }

    let session = session_builder.build().await?;
    session.use_keyspace("equicloud", false).await?;

    let user_hash = crc32fast::hash(user_id.as_bytes());
    let hash_key = format!("settings:{}", user_hash);

    let now = Utc::now().timestamp_millis();

    let query = "INSERT INTO users (id, settings, created_at, updated_at) VALUES (?, ?, ?, ?)";
    session
        .query_unpaged(query, (&hash_key, &settings, now, now))
        .await?;

    Ok(now)
}

async fn delete_user_settings(user_id: &str) -> Result<()> {
    let scylla_uri = env::var("SCYLLA_URI").unwrap_or_default();
    let username = env::var("SCYLLA_USERNAME").ok();
    let password = env::var("SCYLLA_PASSWORD").ok();

    let mut session_builder = SessionBuilder::new().known_node(&scylla_uri);

    if let (Some(user), Some(pass)) = (username, password) {
        session_builder = session_builder.user(&user, &pass);
    }

    let session = session_builder.build().await?;
    session.use_keyspace("equicloud", false).await?;

    let user_hash = crc32fast::hash(user_id.as_bytes());
    let hash_key = format!("settings:{}", user_hash);

    let query = "DELETE FROM users WHERE id = ?";
    session.query_unpaged(query, (&hash_key,)).await?;

    Ok(())
}
