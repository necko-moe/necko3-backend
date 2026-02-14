pub mod token;

pub use token::*;

use crate::config::ChainConfig;
use crate::db::DatabaseAdapter;
use crate::model::{ApiError, ApiResponse, Empty, UpdateChainReq};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/chain",
    request_body = ChainConfig,
    responses(
        (status = 201, description = "Chain added", body = ApiResponse<Empty>),
        (status = 500, description = "Server Error", body = ApiResponse<Empty>)
    ),
    tag = "Chains"
)]
pub async fn add_chain(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChainConfig>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), ApiError> {
    state.db.add_chain(&payload).await
        .map_err(|e| ApiError::InternalServerError(format!("DB Error: {}", e)))?;

    state.start_listening(&payload.name).await
        .map_err(|e| ApiError::InternalServerError(format!("Listener error: {}", e.to_string())))?;

    Ok((StatusCode::CREATED, Json(ApiResponse::ok())))
}

#[utoipa::path(
    get,
    path = "/chain",
    responses(
        (status = 200, description = "Supported chain list", body = ApiResponse<Vec<ChainConfig>>),
        (status = 500, description = "Server Error", body = ApiResponse<Empty>)
    ),
    tag = "Chains"
)]
pub async fn get_chains(
    State(state): State<Arc<AppState>>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<ChainConfig>>>), ApiError> {
    let chains = state.db.get_chains().await
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    Ok((StatusCode::OK, Json(ApiResponse::success(chains))))
}

#[utoipa::path(
    get,
    path = "/chain/{name}",
    params(
        ("name" = String, Path, description = "Chain name")
    ),
    responses(
        (status = 200, description = "Chain configuration", body = ApiResponse<ChainConfig>),
        (status = 404, description = "Chain not found", body = ApiResponse<Empty>),
        (status = 500, description = "Server Error", body = ApiResponse<Empty>)
    ),
    tag = "Chains"
)]
pub async fn get_chain(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<ChainConfig>>), ApiError> {
    let chain = state.db.get_chain(&name).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Chain not found".into()))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(chain))))
}

#[utoipa::path(
    delete,
    path = "/chain/{name}",
    params(
        ("name" = String, Path, description = "Chain name")
    ),
    responses(
        (status = 200, description = "Chain deleted", body = ApiResponse<Empty>),
        (status = 500, description = "Server Error", body = ApiResponse<Empty>)
    ),
    tag = "Chains"
)]
pub async fn delete_chain(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), ApiError> {
    state.stop_listening(&name).await
        .map_err(|e| ApiError::InternalServerError(format!("Listener error: {}", e.to_string())))?;

    state.db.remove_chain(&name).await
        .map_err(|e| ApiError::InternalServerError(format!("DB Error: {}", e)))?;

    Ok((StatusCode::OK, Json(ApiResponse::ok())))
}

#[utoipa::path(
    patch,
    path = "/chain/{name}",
    request_body = UpdateChainReq,
    params(
        ("name" = String, Path, description = "Chain name")
    ),
    responses(
        (status = 200, description = "Chain updated", body = ApiResponse<Empty>),
        (status = 500, description = "Server Error", body = ApiResponse<Empty>)
    ),
    tag = "Chains"
)]
pub async fn update_chain(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<UpdateChainReq>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), ApiError> {
    state.db.update_chain_partial(&name, &payload).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    state.stop_listening(&name).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    state.start_listening(&name).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok((StatusCode::OK, Json(ApiResponse::ok())))
}