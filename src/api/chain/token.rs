use necko3_core::config::TokenConfig;
use necko3_core::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use std::sync::Arc;
use axum::http::StatusCode;
use necko3_core::db::DatabaseAdapter;
use crate::model::{ApiError, ApiResponse, Empty};

#[utoipa::path(
    post,
    path = "/chain/{name}/token",
    params(
        ("name" = String, Path, description = "Network (chain) name")
    ),
    request_body = TokenConfig,
    responses(
        (status = 201, description = "Token added", body = ApiResponse<Empty>),
        (status = 500, description = "Server Error", body = ApiResponse<Empty>)
    ),
    tag = "Tokens"
)]
pub async fn add_token(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(payload): Json<TokenConfig>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), ApiError> {
    state.db.add_token(&name, &payload).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(ApiResponse::ok())))
}

#[utoipa::path(
    get,
    path = "/chain/{name}/token",
    params(
        ("name" = String, Path, description = "Network (chain) name")
    ),
    responses(
        (status = 200, description = "Network's token list", body = ApiResponse<Vec<TokenConfig>>),
        (status = 404, description = "Network (chain) not found", body = ApiResponse<Empty>),
        (status = 500, description = "Server Error", body = ApiResponse<Empty>)
    ),
    tag = "Tokens"
)]
pub async fn get_tokens(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<TokenConfig>>>), ApiError> {
    let tokens = state.db.get_tokens(&name).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::BadRequest(format!("Network '{}' not found", name)))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(tokens))))
}

#[utoipa::path(
    get,
    path = "/chain/{name}/token/{symbol}",
    params(
        ("name" = String, Path, description = "Network (chain) name"),
        ("symbol" = String, Path, description = "Token symbol (e.g. USDC)")
    ),
    responses(
        (status = 200, description = "Token configuration", body = ApiResponse<TokenConfig>),
        (status = 404, description = "Network (chain) or token not found", body = ApiResponse<Empty>),
        (status = 500, description = "Server Error", body = ApiResponse<Empty>)
    ),
    tag = "Tokens"
)]
pub async fn get_token(
    State(state): State<Arc<AppState>>,
    Path((name, symbol)): Path<(String, String)>,
) -> Result<(StatusCode, Json<ApiResponse<TokenConfig>>), ApiError> {
    let token = state.db.get_token(&name, &symbol).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Token or chain not found".into()))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(token))))
}

#[utoipa::path(
    delete,
    path = "/chain/{name}/token/{symbol}",
    params(
        ("name" = String, Path, description = "Network (chain) name"),
        ("symbol" = String, Path, description = "Token symbol (e.g. USDC)")
    ),
    responses(
        (status = 200, description = "Token deleted", body = ApiResponse<Empty>),
        (status = 500, description = "Server Error", body = ApiResponse<Empty>)
    ),
    tag = "Tokens"
)]
pub async fn delete_token(
    State(state): State<Arc<AppState>>,
    Path((name, symbol)): Path<(String, String)>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), ApiError> {
    state.db.remove_token(&name, &symbol).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok((StatusCode::OK, Json(ApiResponse::ok())))
}