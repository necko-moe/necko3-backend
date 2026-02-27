use crate::model::core::{InvoiceStatusSchema, PaymentStatusSchema};
use chrono::{DateTime, Utc};
use necko3_core::model::Invoice;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize)]
pub struct PublicInvoiceModel {
    #[schema(example = "abcef000-abcd-4bcd-8bcd-abcdef000000")]
    pub id: String,
    #[schema(example = "0xabc123...")]
    pub address: String,
    #[schema(example = "25.37")]
    pub amount: String,
    #[schema(example = "0.0")]
    pub paid: String,
    #[schema(example = "USDC")]
    pub token: String,
    #[schema(example = "Polygon")]
    pub network: String,
    #[schema(example = "2026-02-27T21:20:02.537Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2026-02-27T21:35:02.537Z")]
    pub expires_at: DateTime<Utc>,
    pub status: InvoiceStatusSchema,
}

impl From<Invoice> for PublicInvoiceModel {
    fn from(value: Invoice) -> Self {
        Self {
            id: value.id,
            address: value.address,
            amount: value.amount,
            paid: value.paid,
            token: value.token,
            network: value.network,
            created_at: value.created_at,
            expires_at: value.expires_at,
            status: value.status.into(),
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub struct PublicPaymentModel {
    #[schema(example = "abcef000-abcd-4bcd-8bcd-abcdef000000")]
    pub id: String,
    #[schema(example = "abcef000-abcd-4bcd-8bcd-abcdef000000")]
    pub invoice_id: String,
    #[schema(example = "0xabc123...")]
    pub from: String,
    #[schema(example = "0xabc123...")]
    pub to: String,
    #[schema(example = "Polygon")]
    pub network: String,
    #[schema(example = "USDC")]
    pub token: String,
    #[schema(example = "0xabcdef123456...")]
    pub tx_hash: String,
    #[schema(example = "25.37")]
    pub amount: String,
    pub status: PaymentStatusSchema,
    #[schema(example = "2026-02-27T21:20:02.537Z")]
    pub created_at: DateTime<Utc>,
}