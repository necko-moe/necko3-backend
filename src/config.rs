use crate::chain::ChainType;
use alloy::primitives::Address;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct TokenConfig {
    pub symbol: String,
    pub contract: Address,
    pub decimals: u8,
}

#[derive(Debug)]
pub struct ChainConfig {
    pub name: String,
    pub rpc_url: String,
    pub chain_type: ChainType,
    pub native_symbol: String,
    pub decimals: u8,

    pub watch_addresses: RwLock<Vec<Address>>,
    pub tokens: Vec<TokenConfig>,
}