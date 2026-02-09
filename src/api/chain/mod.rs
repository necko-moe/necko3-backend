pub mod token;

pub use token::*;

use crate::config::{ChainConfig, MinChainConfig};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use std::sync::Arc;

pub async fn add_chain(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MinChainConfig>,
) -> String {
    let chain_config: ChainConfig = payload.into();

    match state.add_chain(chain_config).await {
        Ok(_) => "ok".to_owned(),
        Err(e) => format!("error: {}", e),
    }
}

pub async fn get_chains(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<MinChainConfig>> {
    Json(state.get_chains().await)
}

pub async fn get_chain(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Json<MinChainConfig> {
    Json(state.get_chain(&name).await)
}

pub async fn remove_chain(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> String {
    match state.remove_chain(&name).await {
        Ok(_) => "ok".to_owned(),
        Err(e) => format!("error: {}", e),
    }
}