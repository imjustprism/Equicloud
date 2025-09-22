use anyhow::Result;
use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use base64::prelude::*;
use tracing::{error, info};

pub async fn auth_middleware(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let headers = request.headers();
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            error!("No authorization header found");
            StatusCode::UNAUTHORIZED
        })?;

    let token = if auth_header.starts_with("Bearer ") {
        &auth_header[7..]
    } else {
        auth_header
    };

    match verify_token(token).await {
        Ok(user_id) => {
            info!("Successfully authenticated user: {}", user_id);
            request.extensions_mut().insert(user_id);
            Ok(next.run(request).await)
        }
        Err(e) => {
            error!("Token verification failed: {}", e);
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

async fn verify_token(token: &str) -> Result<String> {
    let decoded = BASE64_STANDARD
        .decode(token)
        .map_err(|e| anyhow::anyhow!("Invalid base64 token: {}", e))?;

    let token_str =
        String::from_utf8(decoded).map_err(|e| anyhow::anyhow!("Invalid UTF-8 in token: {}", e))?;

    let parts: Vec<&str> = token_str.split(':').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!(
            "Invalid token format, expected 'secret:userId', got {} parts",
            parts.len()
        ));
    }

    let (_secret, discord_user_id) = (parts[0], parts[1]);

    Ok(discord_user_id.to_string())
}
