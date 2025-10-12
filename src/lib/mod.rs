use anyhow::Result;
use scylla::client::session::Session;
use scylla::client::session_builder::SessionBuilder;
use std::env;

pub mod database;
pub mod hash_migration;
pub mod migrations;
pub mod utils;

pub use database::DatabaseService;
pub use migrations::MigrationRunner;

pub async fn create_database_connection() -> Result<Session> {
    let uri = env::var("SCYLLA_URI").unwrap_or_else(|_| "127.0.0.1:9042".to_string());
    let username = env::var("SCYLLA_USERNAME").ok();
    let password = env::var("SCYLLA_PASSWORD").ok();

    let session_builder = SessionBuilder::new().known_node(uri);

    let session = if let (Some(user), Some(pass)) = (username, password) {
        session_builder.user(user, pass).build().await?
    } else {
        session_builder.build().await?
    };
    Ok(session)
}
