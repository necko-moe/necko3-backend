pub mod core;
pub mod public;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use necko3_core::model::PaginatedVec;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateInvoiceReq {
    #[schema(example = "25.37")]
    pub amount: String,
    #[schema(example = "USDC")]
    pub token: String,
    #[schema(example = "Polygon")]
    pub network: String,
    #[schema(example = "https://merchant.website/payment")]
    pub webhook_url: Option<String>,
    #[schema(example = "mega-secret-random-generated-string")]
    pub webhook_secret: Option<String>,
    /// seconds
    #[schema(example = 900)]
    pub expire_after: Option<u64>, 
}

#[derive(Serialize, ToSchema)]
pub struct Empty {}

#[derive(Serialize, ToSchema)]
pub struct ApiResponse<T> {
    #[schema(example = "success | error")]
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginatedVecPage<T> {
    pub items: Vec<T>,
    #[schema(example = 37)]
    pub total: u64,
    #[schema(example = 20)]
    pub page_size: u32,
    #[schema(example = 1)]
    pub page: u64,
}

impl<T> From<PaginatedVec<T>> for PaginatedVecPage<T> {
    fn from(value: PaginatedVec<T>) -> Self {
        let page = if value.limit > 0 {
            (value.offset / value.limit as u64) + 1
        } else {
            1
        };

        Self {
            items: value.items,
            total: value.total,
            page_size: value.limit,
            page,
        }
    }
}
