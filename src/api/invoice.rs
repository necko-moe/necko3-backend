use necko3_core::chain::{Blockchain, BlockchainAdapter};
use necko3_core::db::DatabaseAdapter;
use necko3_core::model::{Invoice, InvoiceStatus};
use crate::model::{ApiResponse, ApiError, CreateInvoiceReq, Empty};
use necko3_core::state::AppState;
use alloy::primitives::{U256, utils::parse_units};
use axum::extract::{Path, State};
use axum::Json;
use std::sync::Arc;
use axum::http::StatusCode;

#[utoipa::path(
    post,
    path = "/invoice",
    request_body = CreateInvoiceReq,
    responses(
        (status = 201, description = "Invoice created", body = ApiResponse<Invoice>),
        (status = 400, description = "Bad Request", body = ApiResponse<Empty>),
        (status = 404, description = "Chain/token decimals not found", body = ApiResponse<Empty>),
        (status = 500, description = "Server Error", body = ApiResponse<Empty>)
    ),
    tag = "Invoices"
)]
pub async fn create_invoice(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateInvoiceReq>,
) -> Result<(StatusCode, Json<ApiResponse<Invoice>>), ApiError>  {
    let chain_config = state.db.get_chain(&payload.network).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::BadRequest(format!("Network '{}' not supported", payload.network)))?;

    let token_decimals = state.db.get_token_decimals(&payload.network, &payload.token).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::BadRequest(format!("Token '{}' ({}) not supported",
                                                    payload.token, payload.network)))?;

    let amount_raw = parse_units(&payload.amount, token_decimals)
        .map_err(|e| ApiError::BadRequest(format!("Invalid amount format: {}", e)))?;

    let index = state.get_free_slot(&payload.network).await
        .ok_or_else(|| ApiError::InternalServerError("No free slots available".to_owned()))?;

    let blockchain = Blockchain::new(
        state.clone(), chain_config.chain_type, &payload.network, None);
    let address = blockchain.derive_address(index).await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to derive address: {}", e)))?;

    let invoice = Invoice {
        id: uuid::Uuid::new_v4().to_string(),
        address_index: index,
        address: address.clone(),
        amount: payload.amount,
        amount_raw: amount_raw.into(),
        paid: "0".to_string(),
        paid_raw: U256::from(0),
        token: payload.token,
        network: payload.network.clone(),
        decimals: token_decimals,
        created_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::minutes(15),
        status: InvoiceStatus::Pending,
    };

    state.db.add_invoice(&invoice).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    state.db.add_watch_address(&payload.network, &address).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(ApiResponse::success(invoice))))
}

#[utoipa::path(
    get,
    path = "/invoice",
    responses(
        (status = 200, description = "List all invoices", body = ApiResponse<Vec<Invoice>>),
        (status = 500, description = "Server error", body = ApiResponse<Empty>)
    ),
    tag = "Invoices"
)]
pub async fn get_invoices(
    State(state): State<Arc<AppState>>
) -> Result<(StatusCode, Json<ApiResponse<Vec<Invoice>>>), ApiError> {
    let invoices = state.db.get_invoices().await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    Ok((StatusCode::OK, Json(ApiResponse::success(invoices))))
}

#[utoipa::path(
    get,
    path = "/invoice/{id}",
    params(
        ("id" = String, Path, description = "Invoice UUID")
    ),
    responses(
        (status = 200, description = "Invoice data", body = ApiResponse<Invoice>),
        (status = 404, description = "Invoice not found", body = ApiResponse<Empty>),
        (status = 500, description = "Server error", body = ApiResponse<Empty>)
    ),
    tag = "Invoices"
)]
pub async fn get_invoice_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<Invoice>>), ApiError> {
    let invoice = state.db.get_invoice(&id).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .ok_or_else(|| ApiError::NotFound("Invoice not found".into()))?;

    Ok((StatusCode::OK, Json(ApiResponse::success(invoice))))
}

#[utoipa::path(
    delete,
    path = "/invoice/{id}",
    params(
        ("id" = String, Path, description = "Invoice UUID")
    ),
    responses(
        (status = 200, description = "Invoice deleted", body = ApiResponse<Empty>),
        (status = 404, description = "Invoice not found", body = ApiResponse<Empty>),
        (status = 500, description = "Server error", body = ApiResponse<Empty>)
    ),
    tag = "Invoices"
)]
pub async fn delete_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponse<String>>), ApiError> {
    state.db.remove_invoice(&id).await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    Ok((StatusCode::OK, Json(ApiResponse::ok())))
}