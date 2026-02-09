use alloy::primitives::{Address, TxHash, U256};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct PaymentEvent {
    pub network: String,
    pub tx_hash: TxHash,
    pub from: Address,
    pub to: Address,
    pub token: String,
    pub amount: String,
    pub amount_raw: U256,
    pub decimals: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum InvoiceStatus {
    Pending,
    Paid,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Invoice {
    pub id: String,
    pub address_index: u32,
    pub address: Address,
    pub amount: String,
    pub amount_raw: U256,
    pub paid: U256,
    pub token: String,
    pub network: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: InvoiceStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateInvoiceReq {
    pub amount: String,
    pub token: String,
    pub network: String,
}

