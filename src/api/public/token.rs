use crate::model::public::PublicTokenModel;
use crate::model::{ApiError, ApiResponse, Empty};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use necko3_core::db::DatabaseAdapter;
use necko3_core::AppState;
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/public/chain/{name}/token/{symbol}",
    params(
        ("name" = String, Path, description = "Network (chain) name"),
        ("symbol" = String, Path, description = "Token symbol (e.g. USDC)")
    ),
    responses(
        (status = 200, description = "Public token data", body = ApiResponse<PublicTokenModel>),
        (status = 404, description = "Network (chain) or token not found", body = ApiResponse<Empty>),
        (status = 500, description = "Server Error", body = ApiResponse<Empty>)
    ),
    tag = "Public",
    security(
        ()
    )
)]
pub async fn get_public_token(
    State(state): State<Arc<AppState>>,
    Path((name, symbol)): Path<(String, String)>,
) -> Result<(StatusCode, Json<ApiResponse<PublicTokenModel>>), ApiError> {
    let token = state.db.get_token(&name, &symbol).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Network (chain) or token not found".into()))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(token.into()))))
}