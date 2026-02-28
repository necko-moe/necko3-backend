use crate::model::public::{PublicInvoiceModel, PublicPaymentModel};
use crate::model::{ApiError, ApiResponse, Empty, PaginatedVecPage};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use necko3_core::db::DatabaseAdapter;
use necko3_core::deps::format_units;
use necko3_core::AppState;
use std::sync::Arc;
use necko3_core::model::PaymentFilter;
use crate::model::core::PaginationParams;

#[utoipa::path(
    get,
    path = "/public/invoice/{id}",
    params(
        ("id" = String, Path, description = "Invoice UUID")
    ),
    responses(
        (status = 200, description = "Public invoice data", body = ApiResponse<PublicInvoiceModel>),
        (status = 404, description = "Invoice not found", body = ApiResponse<Empty>),
        (status = 500, description = "Server error", body = ApiResponse<Empty>)
    ),
    tag = "Public",
    security(
        ()
    )
)]
pub async fn get_invoice_data(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<PublicInvoiceModel>>), ApiError> {
    let invoice = state.db.get_invoice(&id).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Invoice not found".into()))?;

    let public_invoice: PublicInvoiceModel = invoice.into();

    Ok((StatusCode::OK, Json(ApiResponse::success(public_invoice))))
}

#[utoipa::path(
    get,
    path = "/public/invoice/{id}/payments",
    params(
        ("id" = String, Path, description = "Invoice UUID"),
        PaginationParams
    ),
    responses(
        (status = 200, description = "List all payments for invoice", body = ApiResponse<PaginatedVecPage<PublicPaymentModel>>),
        (status = 500, description = "Server error", body = ApiResponse<Empty>)
    ),
    tag = "Public",
    security(
        ()
    )
)]
pub async fn get_invoice_payments(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(pagination): Query<PaginationParams>,
) -> Result<(StatusCode, Json<ApiResponse<PaginatedVecPage<PublicPaymentModel>>>), ApiError> {
    let filter = PaymentFilter {
        invoice_id: Some(id),
        pagination: pagination.into(),
        ..Default::default()
    };

    let payments = state.db.get_payments(filter).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let mut public_payments = vec![];

    for p in payments.items {
        let chain = p.network;
        let token = p.token;

        let decimals_opt = state.db.get_token_decimals(&chain, &token).await
            .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

        if let Some(decimals) = decimals_opt {
            let amount = format_units(p.amount_raw, decimals)
                .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

            public_payments.push(PublicPaymentModel {
                id: p.id,
                invoice_id: p.invoice_id,
                from: p.from,
                to: p.to,
                network: chain,
                token,
                tx_hash: p.tx_hash,
                amount,
                status: p.status.into(),
                created_at: p.created_at,
            })
        }
    }

    let payments_page: PaginatedVecPage<PublicPaymentModel> = PaginatedVecPage {
        items: public_payments,
        total: payments.total,
        page_size: pagination.page_size,
        page: pagination.page,
    };

    Ok((StatusCode::OK, Json(ApiResponse::success(payments_page))))
}