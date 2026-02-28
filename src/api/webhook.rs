use crate::model::core::WebhookSchema;
use crate::model::core::{PaginationParams, WebhookFilterSchema};
use crate::model::{ApiError, ApiResponse, Empty, PaginatedVecPage};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use necko3_core::db::DatabaseAdapter;
use necko3_core::model::Webhook;
use necko3_core::AppState;
use std::sync::Arc;

#[utoipa::path(
    delete,
    path = "/webhook/{id}",
    params(
        ("id" = String, Path, description = "Webhook UUID")
    ),
    responses(
        (status = 200, description = "Webhook cancelled", body = ApiResponse<Empty>),
        (status = 500, description = "Server error", body = ApiResponse<Empty>)
    ),
    tag = "Webhooks"
)]
pub async fn cancel_webhook(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), ApiError> {
    state.db.cancel_webhook(&id).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok((StatusCode::OK, Json(ApiResponse::ok())))
}

#[utoipa::path(
    get,
    path = "/webhook",
    params(
        WebhookFilterSchema,
        PaginationParams
    ),
    responses(
        (status = 200, description = "List all webhooks", body = ApiResponse<PaginatedVecPage<WebhookSchema>>),
        (status = 500, description = "Server error", body = ApiResponse<Empty>)
    ),
    tag = "Webhooks"
)]
pub async fn get_webhooks(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<WebhookFilterSchema>,
) -> Result<(StatusCode, Json<ApiResponse<PaginatedVecPage<Webhook>>>), ApiError> {
    let webhooks = state.db.get_webhooks(filter.into()).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(webhooks.into()))))
}

#[utoipa::path(
    get,
    path = "/webhook/{id}",
    params(
        ("id" = String, Path, description = "Webhook UUID")
    ),
    responses(
        (status = 200, description = "Webhook data", body = ApiResponse<WebhookSchema>),
        (status = 404, description = "Webhook not found", body = ApiResponse<Empty>),
        (status = 500, description = "Server error", body = ApiResponse<Empty>)
    ),
    tag = "Webhooks"
)]
pub async fn get_webhook(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<Webhook>>), ApiError> {
    let payment = state.db.get_webhook(&id).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Webhook not found".into()))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(payment))))
}