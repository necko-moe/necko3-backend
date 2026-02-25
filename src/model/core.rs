use necko3_core::model::{ChainConfig, ChainType, Invoice, InvoiceStatus, PartialChainUpdate, TokenConfig};
use serde::Serialize;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use necko3_core::deps::U256;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ChainConfigSchema {
    pub name: String,
    pub rpc_url: String,
    pub chain_type: ChainTypeSchema,
    pub xpub: String,
    pub native_symbol: String,
    pub decimals: u8,
    pub last_processed_block: u64,
    pub block_lag: u8,
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
            rpc_url: self.rpc_url,
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
    pub symbol: String,
    pub contract: String,
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
    pub rpc_url: Option<String>,
    pub last_processed_block: Option<u64>,
    pub xpub: Option<String>,
    pub block_lag: Option<u8>,
    pub required_confirmations: Option<u64>,
}

impl Into<PartialChainUpdate> for PartialChainUpdateSchema {
    fn into(self) -> PartialChainUpdate {
        PartialChainUpdate {
            rpc_url: self.rpc_url,
            last_processed_block: self.last_processed_block,
            xpub: self.xpub,
            block_lag: self.block_lag,
            required_confirmations: self.required_confirmations,
        }
    }
}

#[derive(ToSchema)]
pub struct InvoiceSchema {
    pub id: String,
    pub address_index: u32,
    pub address: String,
    pub amount: String,

    #[schema(value_type = String, example = "1000000000000")]
    pub amount_raw: U256,

    pub paid: String,

    #[schema(value_type = String, example = "0")]
    pub paid_raw: U256,

    pub token: String,
    pub network: String,
    pub decimals: u8,
    pub webhook_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub created_at: DateTime<Utc>,
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

#[derive(ToSchema)]
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