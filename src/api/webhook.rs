use crate::model::{ApiError, ApiResponse, Empty, WebhookFilter};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use necko3_core::db::DatabaseAdapter;
use necko3_core::model::Webhook;
use necko3_core::AppState;
use std::sync::Arc;
use crate::model::core::WebhookSchema;

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
        WebhookFilter
    ),
    responses(
        (status = 200, description = "List all webhooks", body = ApiResponse<Vec<WebhookSchema>>),
        (status = 500, description = "Server error", body = ApiResponse<Empty>)
    ),
    tag = "Webhooks"
)]
pub async fn get_webhooks(
    State(state): State<Arc<AppState>>,
    filter: Query<WebhookFilter>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<Webhook>>>), ApiError> {
    // todo multiple query filters at once
    let db_result = if let Some(ref inv) = filter.invoice_id {
        state.db.get_webhooks_by_invoice(inv).await
    } else if let Some(status) = filter.status {
        state.db.get_webhooks_by_status(status.into()).await
    } else if let Some(ref event_type) = filter.event_type {
        state.db.get_webhooks_by_event_type(event_type).await
    } else if let Some(ref url) = filter.url {
        state.db.get_webhooks_by_url(url).await
    } else {
        state.db.get_webhooks().await
    };

    let webhooks = db_result
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(webhooks))))
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