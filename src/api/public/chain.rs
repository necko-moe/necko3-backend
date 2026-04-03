use crate::model::public::PublicChainModel;
use crate::model::{ApiError, ApiResponse, Empty};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use necko3_core::chain::BlockchainAdapter;
use necko3_core::db::DatabaseAdapter;
use necko3_core::AppState;
use std::sync::Arc;

#[utoipa::path(
    get,
    path = "/public/chain/{name}",
    params(
        ("name" = String, Path, description = "Chain name")
    ),
    responses(
        (status = 200, description = "Public chain data", body = ApiResponse<PublicChainModel>),
        (status = 404, description = "Chain not found", body = ApiResponse<Empty>),
        (status = 500, description = "Server Error", body = ApiResponse<Empty>)
    ),
    tag = "Public",
    security(
        ()
    )
)]
pub async fn get_public_chain(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<PublicChainModel>>), ApiError> {
    let chain = state.db.get_chain(&name).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Chain not found".into()))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(chain.config().read().unwrap().clone().into()))))
}