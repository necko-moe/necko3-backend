use serde_aux::prelude::deserialize_option_number_from_string;
use serde_aux::prelude::deserialize_number_from_string;
use necko3_core::model::{
    ChainConfig, ChainType, Invoice, InvoiceFilter, InvoiceStatus, Pagination, PartialChainUpdate,
    Payment, PaymentFilter, PaymentStatus, TokenConfig, WebhookFilter, WebhookStatus
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use necko3_core::deps::U256;
use utoipa::r#gen::serde_json::json;
use utoipa::{IntoParams, ToSchema};

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

impl From<ChainConfigSchema> for ChainConfig {
    fn from(value: ChainConfigSchema) -> Self {
        ChainConfig {
            name: value.name,
            rpc_urls: value.rpc_urls,
            chain_type: value.chain_type.into(),
            xpub: value.xpub,
            native_symbol: value.native_symbol,
            decimals: value.decimals,
            last_processed_block: value.last_processed_block,
            block_lag: value.block_lag,
            required_confirmations: value.required_confirmations,
            watch_addresses: value.watch_addresses,
            tokens: value.tokens,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub enum ChainTypeSchema {
    EVM
}

impl From<ChainTypeSchema> for ChainType {
    fn from(value: ChainTypeSchema) -> Self {
        match value {
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

impl From<TokenConfigSchema> for TokenConfig {
    fn from(value: TokenConfigSchema) -> Self {
        TokenConfig {
            symbol: value.symbol,
            contract: value.contract,
            decimals: value.decimals,
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

impl From<PartialChainUpdateSchema> for PartialChainUpdate {
    fn from(value: PartialChainUpdateSchema) -> Self {
        PartialChainUpdate {
            rpc_urls: value.rpc_urls,
            last_processed_block: value.last_processed_block,
            xpub: value.xpub,
            block_lag: value.block_lag,
            required_confirmations: value.required_confirmations,
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

impl From<InvoiceSchema> for Invoice {
    fn from(value: InvoiceSchema) -> Self {
        Invoice {
            id: value.id,
            address_index: value.address_index,
            address: value.address,
            amount: value.amount,
            amount_raw: value.amount_raw,
            paid: value.paid,
            paid_raw: value.paid_raw,
            token: value.token,
            network: value.network,
            decimals: value.decimals,
            webhook_url: value.webhook_url,
            webhook_secret: value.webhook_secret,
            created_at: value.created_at,
            expires_at: value.expires_at,
            status: value.status.into(),
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

impl From<InvoiceStatusSchema> for InvoiceStatus {
    fn from(value: InvoiceStatusSchema) -> Self {
        match value {
            InvoiceStatusSchema::Pending => InvoiceStatus::Pending,
            InvoiceStatusSchema::Paid => InvoiceStatus::Paid,
            InvoiceStatusSchema::Expired => InvoiceStatus::Expired,
            InvoiceStatusSchema::Cancelled => InvoiceStatus::Cancelled,
        }
    }
}

impl From<InvoiceStatus> for InvoiceStatusSchema {
    fn from(value: InvoiceStatus) -> Self {
        match value {
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

impl From<WebhookStatusSchema> for WebhookStatus {
    fn from(value: WebhookStatusSchema) -> Self {
        match value {
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

impl From<PaymentSchema> for Payment {
    fn from(value: PaymentSchema) -> Self {
        Payment {
            id: value.id,
            invoice_id: value.invoice_id,
            from: value.from,
            to: value.to,
            network: value.network,
            token: value.token,
            tx_hash: value.tx_hash,
            amount_raw: value.amount_raw,
            block_number: value.block_number,
            log_index: value.log_index,
            status: value.status.into(),
            created_at: value.created_at,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, ToSchema)]
pub enum PaymentStatusSchema {
    Confirming,
    Confirmed,
    Cancelled,
}

impl From<PaymentStatusSchema> for PaymentStatus {
    fn from(value: PaymentStatusSchema) -> Self {
        match value {
            PaymentStatusSchema::Confirming => PaymentStatus::Confirming,
            PaymentStatusSchema::Confirmed => PaymentStatus::Confirmed,
            PaymentStatusSchema::Cancelled => PaymentStatus::Cancelled,
        }
    }
}

impl From<PaymentStatus> for PaymentStatusSchema {
    fn from(value: PaymentStatus) -> Self {
        match value {
            PaymentStatus::Confirming => PaymentStatusSchema::Confirming,
            PaymentStatus::Confirmed => PaymentStatusSchema::Confirmed,
            PaymentStatus::Cancelled => PaymentStatusSchema::Cancelled,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PaginationParams {
    #[serde(default = "default_page_size")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[schema(default = 20, example = 20)]
    #[param(default = 20)]
    pub page_size: u32,

    #[serde(default = "default_page")]
    #[serde(deserialize_with = "deserialize_number_from_string")]
    #[schema(default = 1, example = 1)]
    #[param(default = 1)]
    pub page: u64,
}

fn default_page_size() -> u32 { 20 }
fn default_page() -> u64 { 1 }

impl From<PaginationParams> for Pagination {
    fn from(value: PaginationParams) -> Self {
        let page = if value.page == 0 { 1 } else { value.page };

        let limit = value.page_size.min(100);

        Pagination {
            limit,
            offset: (page - 1) * limit as u64,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct InvoiceFilterSchema {
    pub address: Option<String>,
    pub network: Option<String>,
    pub token: Option<String>,
    pub status: Option<InvoiceStatusSchema>,

    #[schema(ignore)]
    #[param(ignore)]
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

impl From<InvoiceFilterSchema> for InvoiceFilter {
    fn from(value: InvoiceFilterSchema) -> Self {
        InvoiceFilter {
            status: value.status.map(|status| status.into()),
            address: value.address,
            network: value.network,
            token: value.token,
            pagination: value.pagination.into(),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PaymentFilterSchema {
    pub invoice_id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub network: Option<String>,
    pub token: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_number_from_string")]
    pub block_number: Option<u64>,
    pub status: Option<PaymentStatusSchema>,

    #[schema(ignore)]
    #[param(ignore)]
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

impl From<PaymentFilterSchema> for PaymentFilter {
    fn from(value: PaymentFilterSchema) -> Self {
        PaymentFilter {
            invoice_id: value.invoice_id,
            from: value.from,
            to: value.to,
            network: value.network,
            token: value.token,
            block_number: value.block_number,
            status: value.status.map(|status| status.into()),
            pagination: value.pagination.into(),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct WebhookFilterSchema {
    pub invoice_id: Option<String>,
    pub event_type: Option<String>,
    pub url: Option<String>,
    pub status: Option<WebhookStatusSchema>,

    #[schema(ignore)]
    #[param(ignore)]
    #[serde(flatten)]
    pub pagination: PaginationParams,
}

impl From<WebhookFilterSchema> for WebhookFilter {
    fn from(value: WebhookFilterSchema) -> Self {
        WebhookFilter {
            invoice_id: value.invoice_id,
            event_type: value.event_type,
            url: value.url,
            status: value.status.map(|status| status.into()),
            pagination: value.pagination.into(),
        }
    }
}