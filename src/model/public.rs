use crate::model::core::{InvoiceStatusSchema, PaymentStatusSchema};
use chrono::{DateTime, Utc};
use necko3_core::model::Invoice;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize)]
pub struct PublicInvoiceModel {
    pub id: String,
    pub address: String,
    pub amount: String,
    pub paid: String,
    pub token: String,
    pub network: String,
    pub created_at: DateTime<Utc>,
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
    pub id: String,
    pub invoice_id: String,
    pub from: String,
    pub to: String,
    pub network: String,
    pub token: String,
    pub tx_hash: String,
    pub amount: String,
    pub status: PaymentStatusSchema,
    pub created_at: DateTime<Utc>,
}