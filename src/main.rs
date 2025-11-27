use axum::http::HeaderValue;
use dotenv::dotenv;
use equicloud::constants::{DEFAULT_HOST, DEFAULT_PORT};
use equicloud::{DatabaseService, MigrationRunner, create_database_connection};
use std::env;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

mod middleware;
mod routes;

fn configure_cors() -> CorsLayer {
    let origins = env::var("CORS_ALLOWED_ORIGINS").ok();

    match origins.as_deref() {
        Some("*") => {
            warn!("CORS configured for all origins - insecure for production!");
            CorsLayer::permissive()
        }
        Some(origins_str) => {
            let valid_origins: Vec<HeaderValue> = origins_str
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();

            if valid_origins.is_empty() {
                warn!("No valid CORS origins found, defaulting to permissive");
                CorsLayer::permissive()
            } else {
                info!("CORS configured for origins: {:?}", valid_origins);
                CorsLayer::new()
                    .allow_origin(valid_origins)
                    .allow_methods(Any)
                    .allow_headers(Any)
            }
        }
        None => {
            warn!("CORS_ALLOWED_ORIGINS not set, using permissive CORS");
            CorsLayer::permissive()
        }
    }
}

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

    let session = match create_database_connection().await {
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
    let migration_runner = MigrationRunner::new(&session);
    if let Err(e) = migration_runner.run_migrations().await {
        error!("Failed to run migrations: {}", e);
        std::process::exit(1);
    }
    info!("Migrations completed");

    let db_service = match DatabaseService::new(session).await {
        Ok(service) => service,
        Err(e) => {
            error!("Failed to create database service: {}", e);
            std::process::exit(1);
        }
    };

    let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or_else(|_| DEFAULT_PORT.to_string());
    let bind_address = format!("{}:{}", server_host, server_port);

    let cors = configure_cors();

    let app = routes::register_routes()
        .layer(axum::extract::Extension(db_service))
        .layer(cors);

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
