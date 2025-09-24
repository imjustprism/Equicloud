use axum::{Extension, extract::Query, response::Json};
use reqwest;
use serde::Deserialize;
use serde_json::{Value, json};
use tracing::{error, info};

use crate::lib::DatabaseService;
use crate::lib::utils::{CONFIG, error_response, get_user_secret};

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

pub async fn oauth_callback(
    Extension(_db): Extension<DatabaseService>,
    Query(params): Query<OAuthCallback>,
) -> Json<Value> {
    if let Some(error) = params.error {
        return Json(error_response(&error));
    }

    let code = match params.code {
        Some(code) => code,
        None => {
            return Json(error_response("Missing code"));
        }
    };

    let redirect_uri = CONFIG.redirect_uri();

    let client = reqwest::Client::new();

    let token_response = client
        .post("https://discord.com/api/oauth2/token")
        .form(&[
            ("client_id", &CONFIG.discord_client_id),
            ("client_secret", &CONFIG.discord_client_secret),
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
            return Json(error_response("Failed to request access token"));
        }
    };

    if !token_response.status().is_success() {
        return Json(error_response("Invalid code"));
    }

    let token_result: DiscordAccessTokenResult = match token_response.json().await {
        Ok(result) => result,
        Err(err) => {
            error!("Failed to parse token response: {}", err);
            return Json(error_response("Failed to parse token response"));
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
            return Json(error_response("Failed to request user"));
        }
    };

    if !user_response.status().is_success() {
        return Json(error_response("Failed to request user"));
    }

    let user_result: DiscordUserResult = match user_response.json().await {
        Ok(result) => result,
        Err(err) => {
            error!("Failed to parse user response: {}", err);
            return Json(error_response("Failed to parse user response"));
        }
    };

    let user_id = user_result.id;

    if let Some(allowed_users) = &CONFIG.discord_allowed_user_ids {
        if !allowed_users.is_empty() {
            let allowed_list: Vec<&str> = allowed_users.split(',').map(|s| s.trim()).collect();
            if !allowed_list.contains(&user_id.as_str()) {
                return Json(error_response("User is not whitelisted"));
            }
        }
    }

    let secret = get_user_secret(&user_id);

    info!("User {} authenticated successfully", user_id);

    Json(json!({
        "secret": secret
    }))
}
