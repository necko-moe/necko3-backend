use necko3_core::model::{ChainConfig, ChainType, Invoice, InvoiceStatus, PartialChainUpdate,
                         Payment, PaymentStatus, TokenConfig, WebhookStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use necko3_core::deps::U256;
use utoipa::r#gen::serde_json::json;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ChainConfigSchema {
    #[schema(example = "Polygon")]
    pub name: String,
    #[schema(example = json!([ "https://rpc-node" ]))]
    pub rpc_urls: Vec<String>,
    pub chain_type: ChainTypeSchema,
    #[schema(example = "xpubabc123...")]
    pub xpub: String,
    #[schema(example = "POL")]
    pub native_symbol: String,
    #[schema(example = "18")]
    pub decimals: u8,
    #[schema(example = "0")]
    /// 0 == use latest block in blockchain
    pub last_processed_block: u64,
    #[schema(example = "5")]
    pub block_lag: u8,
    #[schema(example = "40")]
    pub required_confirmations: u64,

    #[schema(ignore)]
    #[serde(skip)]
    pub watch_addresses: Arc<RwLock<HashSet<String>>>,

    #[schema(ignore)]
    #[serde(skip)]
    pub tokens: Arc<RwLock<HashSet<TokenConfig>>>,
}

impl Into<ChainConfig> for ChainConfigSchema {
    fn into(self) -> ChainConfig {
        ChainConfig {
            name: self.name,
            rpc_urls: self.rpc_urls,
            chain_type: self.chain_type.into(),
            xpub: self.xpub,
            native_symbol: self.native_symbol,
            decimals: self.decimals,
            last_processed_block: self.last_processed_block,
            block_lag: self.block_lag,
            required_confirmations: self.required_confirmations,
            watch_addresses: self.watch_addresses,
            tokens: self.tokens,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub enum ChainTypeSchema {
    EVM
}

impl Into<ChainType> for ChainTypeSchema {
    fn into(self) -> ChainType {
        match self {
            ChainTypeSchema::EVM => ChainType::EVM,
        }
    }
}

#[derive(ToSchema)]
pub struct TokenConfigSchema {
    #[schema(example = "USDC")]
    pub symbol: String,
    #[schema(example = "0xabc123...")]
    pub contract: String,
    #[schema(example = "6")]
    pub decimals: u8,
}

impl Into<TokenConfig> for TokenConfigSchema {
    fn into(self) -> TokenConfig {
        TokenConfig {
            symbol: self.symbol,
            contract: self.contract,
            decimals: self.decimals,
        }
    }
}

#[derive(ToSchema)]
pub struct PartialChainUpdateSchema {
    #[schema(example = json!([ "https://rpc-node" ]))]
    pub rpc_urls: Option<Vec<String>>,
    #[schema(example = "100500")]
    /// 0 == use latest block in blockchain
    pub last_processed_block: Option<u64>,
    #[schema(example = "xpubabc123...")]
    pub xpub: Option<String>,
    #[schema(example = "5")]
    pub block_lag: Option<u8>,
    #[schema(example = "40")]
    pub required_confirmations: Option<u64>,
}

impl Into<PartialChainUpdate> for PartialChainUpdateSchema {
    fn into(self) -> PartialChainUpdate {
        PartialChainUpdate {
            rpc_urls: self.rpc_urls,
            last_processed_block: self.last_processed_block,
            xpub: self.xpub,
            block_lag: self.block_lag,
            required_confirmations: self.required_confirmations,
        }
    }
}

#[derive(ToSchema)]
pub struct InvoiceSchema {
    #[schema(example = "abcef000-abcd-4bcd-8bcd-abcdef000000")]
    pub id: String,
    #[schema(example = 0)]
    pub address_index: u32,
    #[schema(example = "0xabc123...")]
    pub address: String,
    #[schema(example = "25.37")]
    pub amount: String,
    #[schema(value_type = String, example = "0x1831d90")]
    pub amount_raw: U256,
    #[schema(example = "0.0")]
    pub paid: String,
    #[schema(value_type = String, example = "0x0")]
    pub paid_raw: U256,
    #[schema(example = "USDC")]
    pub token: String,
    #[schema(example = "Polygon")]
    pub network: String,
    #[schema(example = 6)]
    pub decimals: u8,
    #[schema(example = "https://merchant.website/payment")]
    pub webhook_url: Option<String>,
    #[schema(example = "mega-secret-random-generated-string")]
    pub webhook_secret: Option<String>,
    #[schema(example = "2026-02-27T21:20:02.537Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2026-02-27T21:35:02.537Z")]
    pub expires_at: DateTime<Utc>,
    pub status: InvoiceStatusSchema,
}

impl Into<Invoice> for InvoiceSchema {
    fn into(self) -> Invoice {
        Invoice {
            id: self.id,
            address_index: self.address_index,
            address: self.address,
            amount: self.amount,
            amount_raw: self.amount_raw,
            paid: self.paid,
            paid_raw: self.paid_raw,
            token: self.token,
            network: self.network,
            decimals: self.decimals,
            webhook_url: self.webhook_url,
            webhook_secret: self.webhook_secret,
            created_at: self.created_at,
            expires_at: self.expires_at,
            status: self.status.into(),
        }
    }
}

#[derive(ToSchema, Serialize, Deserialize)]
pub enum InvoiceStatusSchema {
    Pending,
    Paid,
    Expired,
    Cancelled,
}

impl Into<InvoiceStatus> for InvoiceStatusSchema {
    fn into(self) -> InvoiceStatus {
        match self {
            InvoiceStatusSchema::Pending => InvoiceStatus::Pending,
            InvoiceStatusSchema::Paid => InvoiceStatus::Paid,
            InvoiceStatusSchema::Expired => InvoiceStatus::Expired,
            InvoiceStatusSchema::Cancelled => InvoiceStatus::Cancelled,
        }
    }
}

impl Into<InvoiceStatusSchema> for InvoiceStatus {
    fn into(self) -> InvoiceStatusSchema {
        match self {
            InvoiceStatus::Pending => InvoiceStatusSchema::Pending,
            InvoiceStatus::Paid => InvoiceStatusSchema::Paid,
            InvoiceStatus::Expired => InvoiceStatusSchema::Expired,
            InvoiceStatus::Cancelled => InvoiceStatusSchema::Cancelled,
        }
    }
}

#[derive(ToSchema)]
pub struct WebhookSchema {
    #[schema(example = "abcef000-abcd-4bcd-8bcd-abcdef000000")]
    pub id: String,
    #[schema(example = "abcef000-abcd-4bcd-8bcd-abcdef000000")]
    pub invoice_id: String,
    #[schema(example = "https://merchant.website/payment")]
    pub url: String,
    pub payload: WebhookEventSchema,
    pub status: WebhookStatusSchema,
    #[schema(example = 3)]
    pub attempts: u32,
    #[schema(example = 5)]
    pub max_retries: u32,
    #[schema(example = "2026-02-27T21:26:02.537Z")]
    pub next_retry: DateTime<Utc>,
    #[schema(example = "2026-02-27T21:25:02.537Z")]
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
#[serde(tag = "event_type", content = "data", rename_all = "snake_case")]
pub enum WebhookEventSchema {
    TxDetected {
        #[schema(example = "abcef000-abcd-4bcd-8bcd-abcdef000000")]
        invoice_id: String,
        #[schema(example = "0xabcdef123456...")]
        tx_hash: String,
        #[schema(example = "3.7")]
        amount: String,
        #[schema(example = "USDC")]
        currency: String,
    },
    TxConfirmed {
        #[schema(example = "abcef000-abcd-4bcd-8bcd-abcdef000000")]
        invoice_id: String,
        #[schema(example = "0xabcdef123456...")]
        tx_hash: String,
        #[schema(example = 40)]
        confirmations: u64,
    },
    InvoicePaid {
        #[schema(example = "abcef000-abcd-4bcd-8bcd-abcdef000000")]
        invoice_id: String,
        #[schema(example = "25.37")]
        paid_amount: String,
    },
    InvoiceExpired {
        #[schema(example = "abcef000-abcd-4bcd-8bcd-abcdef000000")]
        invoice_id: String,
    },
}

#[derive(Copy, Clone, Serialize, Deserialize, ToSchema)]
pub enum WebhookStatusSchema {
    Pending,
    Processing,
    Sent,
    Failed,
    Cancelled,
}

impl Into<WebhookStatus> for WebhookStatusSchema {
    fn into(self) -> WebhookStatus {
        match self {
            WebhookStatusSchema::Pending => WebhookStatus::Pending,
            WebhookStatusSchema::Processing => WebhookStatus::Processing,
            WebhookStatusSchema::Sent => WebhookStatus::Sent,
            WebhookStatusSchema::Failed => WebhookStatus::Failed,
            WebhookStatusSchema::Cancelled => WebhookStatus::Cancelled,
        }
    }
}

#[derive(ToSchema, Serialize)]
pub struct PaymentSchema {
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

    #[schema(value_type = String, example = "0x1831d90")]
    pub amount_raw: U256,
    #[schema(example = "100500")]
    pub block_number: u64,
    #[schema(example = "37")]
    pub log_index: u64,
    pub status: PaymentStatusSchema,
    #[schema(example = "2026-02-27T21:20:02.537Z")]
    pub created_at: DateTime<Utc>,
}

impl Into<Payment> for PaymentSchema {
    fn into(self) -> Payment {
        Payment {
            id: self.id,
            invoice_id: self.invoice_id,
            from: self.from,
            to: self.to,
            network: self.network,
            token: self.token,
            tx_hash: self.tx_hash,
            amount_raw: self.amount_raw,
            block_number: self.block_number,
            log_index: self.log_index,
            status: self.status.into(),
            created_at: self.created_at,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, ToSchema)]
pub enum PaymentStatusSchema {
    Confirming,
    Confirmed,
    Cancelled,
}

impl Into<PaymentStatus> for PaymentStatusSchema {
    fn into(self) -> PaymentStatus {
        match self {
            PaymentStatusSchema::Confirming => PaymentStatus::Confirming,
            PaymentStatusSchema::Confirmed => PaymentStatus::Confirmed,
            PaymentStatusSchema::Cancelled => PaymentStatus::Cancelled,
        }
    }
}

impl Into<PaymentStatusSchema> for PaymentStatus {
    fn into(self) -> PaymentStatusSchema {
        match self {
            PaymentStatus::Confirming => PaymentStatusSchema::Confirming,
            PaymentStatus::Confirmed => PaymentStatusSchema::Confirmed,
            PaymentStatus::Cancelled => PaymentStatusSchema::Cancelled,
        }
    }
}