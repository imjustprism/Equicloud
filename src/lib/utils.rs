use once_cell::sync::Lazy;
use serde_json::{Value, json};
use std::env;

pub fn hash_user_id(user_id: &str) -> String {
    let user_hash = crc32fast::hash(user_id.as_bytes());
    format!("settings:{}", user_hash)
}

pub fn get_user_secret(user_id: &str) -> String {
    let user_hash = crc32fast::hash(user_id.as_bytes());
    format!("{:08x}", user_hash)
}

#[derive(Clone)]
pub struct Config {
    pub max_backup_size_bytes: usize,
    pub discord_client_id: String,
    pub discord_client_secret: String,
    pub server_fqdn: String,
    pub discord_allowed_user_ids: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            max_backup_size_bytes: env::var("MAX_BACKUP_SIZE_BYTES")
                .unwrap_or_else(|_| "62914560".to_string())
                .parse::<usize>()
                .unwrap_or(62914560),
            discord_client_id: env::var("DISCORD_CLIENT_ID").unwrap_or_default(),
            discord_client_secret: env::var("DISCORD_CLIENT_SECRET").unwrap_or_default(),
            server_fqdn: env::var("SERVER_FQDN").unwrap_or_default(),
            discord_allowed_user_ids: env::var("DISCORD_ALLOWED_USER_IDS").ok(),
        }
    }

    pub fn redirect_uri(&self) -> String {
        format!("{}/v1/oauth/callback", self.server_fqdn)
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(Config::from_env);

pub fn error_response(message: &str) -> Value {
    json!({
        "error": message
    })
}
