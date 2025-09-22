#![allow(special_module_name)] // for lib

use dotenv::dotenv;
use std::env;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use tracing_subscriber;

mod lib;
mod middleware;
mod routes;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting EquiCloud server");
    info!("Connecting to database...");

    let session = match lib::create_database_connection().await {
        Ok(session) => {
            info!("Database connection successful");
            session
        }
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };

    info!("Running migrations...");
    let migration_runner = lib::MigrationRunner::new(session);
    if let Err(e) = migration_runner.run_migrations().await {
        error!("Failed to run migrations: {}", e);
        std::process::exit(1);
    }
    info!("Migrations completed");

    let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("{}:{}", server_host, server_port);

    let app = routes::register_routes().layer(CorsLayer::permissive());

    let listener = TcpListener::bind(&bind_address)
        .await
        .expect("Failed to bind to address");

    info!("Server running on http://{}", bind_address);

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
