use alloy::primitives::{TxHash, U256};
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};
use utoipa::ToSchema;

#[derive(Debug, Clone)]
pub struct PaymentEvent {
    pub network: String,
    pub tx_hash: TxHash,
    pub from: String,
    pub to: String,
    pub token: String,
    pub amount: String,
    pub amount_raw: U256,
    pub decimals: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, ToSchema,
    Display, EnumString, AsRefStr)]
#[strum(serialize_all = "PascalCase")]
pub enum InvoiceStatus {
    Pending,
    Paid,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Invoice {
    pub id: String,
    pub address_index: u32,
    pub address: String,
    pub amount: String,
    #[schema(value_type = String, example = "1000000000000000000")]
    pub amount_raw: U256,
    pub paid: String,
    #[schema(value_type = String, example = "0")]
    pub paid_raw: U256,
    pub token: String,
    pub network: String,
    pub decimals: u8,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: InvoiceStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateInvoiceReq {
    pub amount: String,
    pub token: String,
    pub network: String,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateChainReq {
    pub rpc_url: Option<String>,
    pub last_processed_block: Option<u64>,
    pub xpub: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct Empty {}

#[derive(Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            status: "success".to_string(),
            data: Some(data),
            message: None,
        }
    }

    pub fn ok() -> Self {
        Self {
            status: "success".to_string(),
            data: None,
            message: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            status: "error".to_string(),
            data: None,
            message: Some(msg.into()),
        }
    }
}

pub enum ApiError {
    BadRequest(String),
    NotFound(String),
    InternalServerError(String),
}

impl<E> From<E> for ApiError
where
    E: std::error::Error,
{
    fn from(err: E) -> Self {
        Self::InternalServerError(err.to_string())
    }
}

// Самое главное: превращаем AppError в HTTP ответ
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = ApiResponse::<()>::error(msg);
        (status, Json(body)).into_response()
    }
}