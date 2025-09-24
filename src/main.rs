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

    info!("Running migrations...");
    let migration_session = match lib::create_database_connection().await {
        Ok(session) => session,
        Err(e) => {
            error!("Failed to create migration connection: {}", e);
            std::process::exit(1);
        }
    };
    let migration_runner = lib::MigrationRunner::new(migration_session);
    if let Err(e) = migration_runner.run_migrations().await {
        error!("Failed to run migrations: {}", e);
        std::process::exit(1);
    }
    info!("Migrations completed");

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

    let db_service = match lib::DatabaseService::new(session).await {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to create database service: {}", e);
            std::process::exit(1);
        }
    };

    let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_address = format!("{}:{}", server_host, server_port);

    let app = routes::register_routes()
        .layer(axum::extract::Extension(db_service))
        .layer(CorsLayer::permissive());

    let listener = TcpListener::bind(&bind_address).await.unwrap_or_else(|e| {
        error!("Failed to bind to address {}: {}", bind_address, e);
        std::process::exit(1);
    });

    info!("Server running on http://{}", bind_address);

    if let Err(e) = axum::serve(listener, app).await {
        error!("Server failed to start: {}", e);
        std::process::exit(1);
    }
}
