use crate::chain::ChainType;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::RwLock;

#[derive(Debug, Clone, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct TokenConfig {
    pub symbol: String,
    pub contract: String,
    pub decimals: u8,
}

#[derive(Debug)]
pub struct ChainConfig {
    pub name: String,
    pub rpc_url: String,
    pub chain_type: ChainType,
    pub xpub: String,
    pub native_symbol: String,
    pub decimals: u8,

    pub watch_addresses: RwLock<HashSet<String>>,
    pub tokens: RwLock<HashSet<TokenConfig>>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinChainConfig {
    pub name: String,
    pub rpc_url: String,
    pub chain_type: ChainType,
    pub xpub: String,
    pub native_symbol: String,
    pub decimals: u8,
}

impl Into<ChainConfig> for MinChainConfig {
    fn into(self) -> ChainConfig {
        ChainConfig {
            name: self.name,
            rpc_url: self.rpc_url,
            chain_type: self.chain_type,
            xpub: self.xpub,
            native_symbol: self.native_symbol,
            decimals: self.decimals,
            watch_addresses: RwLock::new(HashSet::new()),
            tokens: RwLock::new(HashSet::new()),
        }
    }
}

impl Into<MinChainConfig> for ChainConfig {
    fn into(self) -> MinChainConfig {
        MinChainConfig {
            name: self.name,
            rpc_url: self.rpc_url,
            chain_type: self.chain_type,
            xpub: self.xpub,
            native_symbol: self.native_symbol,
            decimals: self.decimals
        }
    }
}

impl Into<ChainConfig> for &MinChainConfig {
    fn into(self) -> ChainConfig {
        ChainConfig {
            name: self.name.clone(),
            rpc_url: self.rpc_url.clone(),
            chain_type: self.chain_type,
            xpub: self.xpub.clone(),
            native_symbol: self.native_symbol.clone(),
            decimals: self.decimals,
            watch_addresses: RwLock::new(HashSet::new()),
            tokens: RwLock::new(HashSet::new()),
        }
    }
}

impl Into<MinChainConfig> for &ChainConfig {
    fn into(self) -> MinChainConfig {
        MinChainConfig {
            name: self.name.clone(),
            rpc_url: self.rpc_url.clone(),
            chain_type: self.chain_type,
            xpub: self.xpub.clone(),
            native_symbol: self.native_symbol.clone(),
            decimals: self.decimals
        }
    }
}
