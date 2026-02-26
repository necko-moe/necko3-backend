pub mod core;
pub mod public;

use crate::model::core::{PaymentStatusSchema, WebhookStatusSchema};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateInvoiceReq {
    pub amount: String,
    pub token: String,
    pub network: String,
    pub webhook_url: Option<String>,
    pub webhook_secret: Option<String>,
    /// seconds
    pub expire_after: Option<u64>, 
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PaymentFilter {
    pub invoice_id: Option<String>,
    pub status: Option<PaymentStatusSchema>,
    pub network: Option<String>,
    pub address_to: Option<String>
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct WebhookFilter {
    pub invoice_id: Option<String>,
    pub status: Option<WebhookStatusSchema>,
    pub event_type: Option<String>,
    pub url: Option<String>
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