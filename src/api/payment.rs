use crate::model::{ApiError, ApiResponse, Empty, PaymentFilter};
use crate::model::core::PaymentSchema;
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
        PaymentFilter
    ),
    responses(
        (status = 200, description = "List all payments", body = ApiResponse<Vec<PaymentSchema>>),
        (status = 500, description = "Server error", body = ApiResponse<Empty>)
    ),
    tag = "Payments"
)]
pub async fn get_payments(
    State(state): State<Arc<AppState>>,
    filter: Query<PaymentFilter>,
) -> Result<(StatusCode, Json<ApiResponse<Vec<Payment>>>), ApiError> {
    // todo multiple query filters at once
    let db_result = if let Some(ref inv) = filter.invoice_id {
        state.db.get_payments_by_invoice(inv).await
    } else if let Some(status) = filter.status {
        state.db.get_payments_by_status(status.into()).await
    } else if let Some(ref net) = filter.network {
        state.db.get_payments_by_network(net).await
    } else if let Some(ref addr) = filter.address_to {
        state.db.get_payments_by_address(addr).await
    } else {
        state.db.get_payments().await
    };

    let payments = db_result
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(payments))))
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