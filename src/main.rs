mod api;
mod model;

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
            .unwrap_or(20);

        Database::init(&database_url, max_connections, &db_type).await?
    };

    api::serve({
        let api_key = std::env::var("API_KEY")
            .expect("API_KEY must be set");

        match AppState::init(db, &api_key, Duration::from_secs(30)).await {
            Ok(state) => state,
            Err(e) => panic!("Failed to init AppState: {}", e),
        }
    }).await?;

    Ok(())
}