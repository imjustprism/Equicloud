use anyhow::Result;
use axum::{extract::Query, response::Json};
use reqwest;
use scylla::client::session_builder::SessionBuilder;
use serde::Deserialize;
use serde_json::{Value, json};
use std::env;
use tracing::{error, info};

#[derive(Deserialize)]
pub struct OAuthCallback {
    pub code: Option<String>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
struct DiscordAccessTokenResult {
    access_token: String,
}

#[derive(Deserialize)]
struct DiscordUserResult {
    id: String,
}

pub async fn oauth_callback(Query(params): Query<OAuthCallback>) -> Json<Value> {
    if let Some(error) = params.error {
        return Json(json!({
            "error": error
        }));
    }

    let code = match params.code {
        Some(code) => code,
        None => {
            return Json(json!({
                "error": "Missing code"
            }));
        }
    };

    let client_id = env::var("DISCORD_CLIENT_ID").unwrap_or_default();
    let client_secret = env::var("DISCORD_CLIENT_SECRET").unwrap_or_default();
    let server_fqdn = env::var("SERVER_FQDN").unwrap_or_default();
    let redirect_uri = format!("{}/v1/oauth/callback", server_fqdn);

    let client = reqwest::Client::new();

    let token_response = client
        .post("https://discord.com/api/oauth2/token")
        .form(&[
            ("client_id", &client_id),
            ("client_secret", &client_secret),
            ("grant_type", &"authorization_code".to_string()),
            ("code", &code),
            ("redirect_uri", &redirect_uri),
            ("scope", &"identify".to_string()),
        ])
        .send()
        .await;

    let token_response = match token_response {
        Ok(response) => response,
        Err(err) => {
            error!("Failed to request access token: {}", err);
            return Json(json!({
                "error": "Failed to request access token"
            }));
        }
    };

    if !token_response.status().is_success() {
        return Json(json!({
            "error": "Invalid code"
        }));
    }

    let token_result: DiscordAccessTokenResult = match token_response.json().await {
        Ok(result) => result,
        Err(err) => {
            error!("Failed to parse token response: {}", err);
            return Json(json!({
                "error": "Failed to parse token response"
            }));
        }
    };

    let user_response = client
        .get("https://discord.com/api/users/@me")
        .header(
            "Authorization",
            format!("Bearer {}", token_result.access_token),
        )
        .send()
        .await;

    let user_response = match user_response {
        Ok(response) => response,
        Err(err) => {
            error!("Failed to request user: {}", err);
            return Json(json!({
                "error": "Failed to request user"
            }));
        }
    };

    if !user_response.status().is_success() {
        return Json(json!({
            "error": "Failed to request user"
        }));
    }

    let user_result: DiscordUserResult = match user_response.json().await {
        Ok(result) => result,
        Err(err) => {
            error!("Failed to parse user response: {}", err);
            return Json(json!({
                "error": "Failed to parse user response"
            }));
        }
    };

    let user_id = user_result.id;

    if let Ok(allowed_users) = env::var("DISCORD_ALLOWED_USER_IDS") {
        if !allowed_users.is_empty() {
            let allowed_list: Vec<&str> = allowed_users.split(',').map(|s| s.trim()).collect();
            if !allowed_list.contains(&user_id.as_str()) {
                return Json(json!({
                    "error": "User is not whitelisted"
                }));
            }
        }
    }

    let secret = match get_or_create_user_secret(&user_id).await {
        Ok(secret) => secret,
        Err(err) => {
            error!("Failed to get/create user secret: {}", err);
            return Json(json!({
                "error": "Failed to generate secret"
            }));
        }
    };

    info!("User {} authenticated successfully", user_id);

    Json(json!({
        "secret": secret
    }))
}

async fn get_or_create_user_secret(user_id: &str) -> Result<String> {
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
    let secret = format!("{:08x}", user_hash);

    Ok(secret)
}
