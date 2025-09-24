use crate::lib::utils::CONFIG;
use axum::response::Json;
use serde_json::{Value, json};

pub async fn oauth_settings() -> Json<Value> {
    Json(json!({
        "clientId": CONFIG.discord_client_id,
        "redirectUri": CONFIG.redirect_uri()
    }))
}
