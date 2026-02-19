mod api;
mod model;

use std::env;
use necko3_core::db::Database;
use necko3_core::state::AppState;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let db: Database = {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let db_type = std::env::var("DATABASE_TYPE")
            .expect("DATABASE_TYPE must be set");

        let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "20".into())
            .parse::<u32>()
            .expect("Failed to parse DATABASE_MAX_CONNECTIONS as number u32");

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
    
    api::serve({
        let api_key = std::env::var("API_KEY")
            .expect("API_KEY must be set");

        match AppState::init(db, &api_key, Duration::from_secs(janitor_interval), 
                             Duration::from_secs(confirmator_interval)).await {
            Ok(state) => state,
            Err(e) => panic!("Failed to init AppState: {}", e),
        }
    }, include_swagger, api::cors_from_str(&cors_origins)).await?;

    Ok(())
}