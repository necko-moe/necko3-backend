use crate::config::TokenConfig;
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use std::sync::Arc;

pub async fn add_token(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<TokenConfig>,
) -> String {
    match state.add_token(name, payload).await {
        Ok(_) => "ok".to_owned(),
        Err(e) => format!("error: {}", e),
    }
}

pub async fn get_tokens(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<Vec<TokenConfig>>, String> {
    match state.get_tokens(name).await {
        Ok(vec) => Ok(Json(vec)),
        Err(e) => Err(format!("{}", e)),
    }
}

pub async fn get_token(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Path(symbol): Path<String>,
) -> Result<Json<TokenConfig>, String> {
    match state.get_token(name, symbol).await {
        Ok(conf) => Ok(Json(conf)),
        Err(e) => Err(format!("{}", e)),
    }
}

pub async fn remove_token(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Path(symbol): Path<String>,
) -> String {
    match state.remove_token(name, symbol).await {
        Ok(_) => "ok".to_owned(),
        Err(e) => format!("error: {}", e),
    }
}