use axum::response::Json;
use equicloud::utils::CONFIG;
use serde_json::{Value, json};

pub async fn oauth_settings() -> Json<Value> {
    Json(json!({
        "clientId": CONFIG.discord_client_id,
        "redirectUri": CONFIG.redirect_uri()
    }))
}
