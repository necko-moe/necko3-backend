use crate::model::core::{PaginationParams, PaymentFilterSchema, PaymentSchema};
use crate::model::{ApiError, ApiResponse, Empty, PaginatedVecPage};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use necko3_core::db::DatabaseAdapter;
use necko3_core::model::Payment;
use necko3_core::AppState;
use std::sync::Arc;

#[utoipa::path(
    delete,
    path = "/payment/{id}",
    params(
        ("id" = String, Path, description = "Payment UUID")
    ),
    responses(
        (status = 200, description = "Payment cancelled", body = ApiResponse<Empty>),
        (status = 500, description = "Server error", body = ApiResponse<Empty>)
    ),
    tag = "Payments"
)]
pub async fn cancel_payment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), ApiError> {
    state.db.cancel_payment(&id).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok((StatusCode::OK, Json(ApiResponse::ok())))
}

#[utoipa::path(
    get,
    path = "/payment",
    params(
        PaymentFilterSchema,
        PaginationParams
    ),
    responses(
        (status = 200, description = "List all payments", body = ApiResponse<PaginatedVecPage<PaymentSchema>>),
        (status = 500, description = "Server error", body = ApiResponse<Empty>)
    ),
    tag = "Payments"
)]
pub async fn get_payments(
    State(state): State<Arc<AppState>>,
    Query(filter): Query<PaymentFilterSchema>,
) -> Result<(StatusCode, Json<ApiResponse<PaginatedVecPage<Payment>>>), ApiError> {
    let payments = state.db.get_payments(filter.into()).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(payments.into()))))
}

#[utoipa::path(
    get,
    path = "/payment/{id}",
    params(
        ("id" = String, Path, description = "Payment UUID")
    ),
    responses(
        (status = 200, description = "Payment data", body = ApiResponse<PaymentSchema>),
        (status = 404, description = "Payment not found", body = ApiResponse<Empty>),
        (status = 500, description = "Server error", body = ApiResponse<Empty>)
    ),
    tag = "Payments"
)]
pub async fn get_payment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<Payment>>), ApiError> {
    let payment = state.db.get_payment(&id).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Payment not found".into()))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(payment))))
}