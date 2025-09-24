use axum::{
    Extension, Router,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
};
use serde_json::json;
use std::env;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::lib::DatabaseService;

static START_TIME: OnceLock<u64> = OnceLock::new();

pub fn register() -> Router {
    START_TIME.get_or_init(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    Router::new().route("/metrics", get(get_metrics))
}

async fn get_metrics(Extension(db): Extension<DatabaseService>) -> impl IntoResponse {
    let metrics_enabled = env::var("METRICS_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    if !metrics_enabled {
        return StatusCode::NOT_FOUND.into_response();
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
        .as_secs();

    let start_time = *START_TIME.get().unwrap_or(&0);
    let uptime = now - start_time;

    let user_counts = match get_user_counts(&db).await {
        Ok(counts) => counts,
        Err(_) => UserCounts::default(),
    };

    Json(json!({
        "uptime_seconds": uptime,
        "users_total": user_counts.total,
        "users_day": user_counts.day,
        "users_week": user_counts.week,
        "users_month": user_counts.month,
        "database_connected": true,
        "timestamp": chrono::Utc::now().timestamp()
    }))
    .into_response()
}

#[derive(Default)]
struct UserCounts {
    total: u64,
    week: u64,
    month: u64,
    day: u64,
}

async fn get_user_counts(db: &DatabaseService) -> Result<UserCounts, anyhow::Error> {
    let now = chrono::Utc::now().timestamp_millis();
    let day_ago = now - (24 * 60 * 60 * 1000);
    let week_ago = now - (7 * 24 * 60 * 60 * 1000);
    let month_ago = now - (30 * 24 * 60 * 60 * 1000);

    let total = query_user_count(db, "SELECT COUNT(*) FROM users", &[]).await?;
    let day = query_user_count_with_filter(db, day_ago, "last day").await?;
    let week = query_user_count_with_filter(db, week_ago, "last week").await?;
    let month = query_user_count_with_filter(db, month_ago, "last month").await?;

    Ok(UserCounts {
        total,
        day,
        week,
        month,
    })
}

async fn query_user_count(
    db: &DatabaseService,
    query: &str,
    params: &[i64],
) -> Result<u64, anyhow::Error> {
    let result = db
        .session()
        .query_unpaged(query, params)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to query users: {}", e))?;

    let count = result
        .into_rows_result()?
        .rows::<(i64,)>()?
        .next()
        .transpose()?
        .map(|row| row.0 as u64)
        .unwrap_or(0);

    Ok(count)
}

async fn query_user_count_with_filter(
    db: &DatabaseService,
    timestamp: i64,
    period: &str,
) -> Result<u64, anyhow::Error> {
    let query = "SELECT COUNT(*) FROM users WHERE created_at > ? ALLOW FILTERING";
    let count = db
        .session()
        .query_unpaged(query, (timestamp,))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to query users in {}: {}", period, e))?
        .into_rows_result()?
        .rows::<(i64,)>()?
        .next()
        .transpose()?
        .map(|row| row.0 as u64)
        .unwrap_or(0);

    Ok(count)
}
