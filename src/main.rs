mod api;
mod model;

use std::env;
use necko3_core::db::Database;
use necko3_core::state::AppState;
use std::time::Duration;

use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let log_format = std::env::var("LOG_FORMAT")
        .unwrap_or_else(|_| "full".to_string());
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info".into());

    let fmt_layer = tracing_subscriber::fmt::layer();

    match log_format.as_str() {
        "compact" => {
            tracing_subscriber::registry()
                .with(fmt_layer.compact().with_target(false))
                .with(filter)
                .init();
        }
        "json" => {
            tracing_subscriber::registry()
                .with(fmt_layer.json())
                .with(filter)
                .init();
        }
        _ => {
            tracing_subscriber::registry()
                .with(fmt_layer)
                .with(filter)
                .init();
        }
    };

    info!("Initializing application...");

    info!("Initializing application...");

    let db: Database = {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let db_type = std::env::var("DATABASE_TYPE")
            .expect("DATABASE_TYPE must be set");

        let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "20".into())
            .parse::<u32>()
            .expect("Failed to parse DATABASE_MAX_CONNECTIONS as number u32");

        info!(db_type, max_connections, "Connecting to database...");
        Database::init(&database_url, max_connections, &db_type).await?
    };

    let include_swagger = std::env::var("INCLUDE_SWAGGER")
        .unwrap_or_else(|_| "false".into())
        .parse::<bool>()
        .expect("Failed to parse INCLUDE_SWAGGER as boolean");

    let janitor_interval: u64 = env::var("JANITOR_INTERVAL")
        .unwrap_or_else(|_| "30".into())
        .parse::<u64>()
        .expect("Failed to parse JANITOR_INTERVAL as number u64");

    let confirmator_interval: u64 = env::var("CONFIRMATOR_INTERVAL")
        .unwrap_or_else(|_| "5".into())
        .parse::<u64>()
        .expect("Failed to parse CONFIRMATOR_INTERVAL as number u64");

    let cors_origins = env::var("CORS_ALLOWED_ORIGINS")
        .expect("CORS_ALLOWED_ORIGINS must be set");

    info!(
        janitor_sec = janitor_interval,
        confirmator_sec = confirmator_interval,
        swagger = include_swagger,
        "Configuration loaded"
    );

    let api_key = std::env::var("API_KEY")
        .expect("API_KEY must be set");

    let state = match AppState::init(db, &api_key, Duration::from_secs(janitor_interval),
                         Duration::from_secs(confirmator_interval)).await {
        Ok(state) => state,
        Err(e) => {
            error!(error = %e, "Failed to init AppState");
            panic!("Failed to init AppState: {}", e);
        },
    };

    let bind_address = std::env::var("BIND_ADDRESS")
        .unwrap_or_else(|_| "127.0.0.1:3000".into());

    api::serve(state, include_swagger, api::cors_from_str(&cors_origins), &bind_address).await?;

    Ok(())
}