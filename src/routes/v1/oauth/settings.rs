use axum::response::Json;
use serde_json::{json, Value};
use std::env;

pub async fn oauth_settings() -> Json<Value> {
    let client_id = env::var("DISCORD_CLIENT_ID").unwrap_or_default();
    let server_fqdn = env::var("SERVER_FQDN").unwrap_or_default();
    let redirect_uri = format!("{}/v1/oauth/callback", server_fqdn);

    Json(json!({
        "clientId": client_id,
        "redirectUri": redirect_uri
    }))
}