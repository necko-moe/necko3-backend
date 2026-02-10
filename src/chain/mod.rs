use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use tokio::sync::mpsc;
use crate::chain::Blockchain::Evm;
use crate::chain::evm::EvmBlockchain;
use crate::config::ChainConfig;
use crate::model::PaymentEvent;

pub mod evm;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ChainType {
    EVM
}

impl Display for ChainType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainType::EVM => write!(f, "EVM"),
        }
    }
}

pub trait BlockchainAdapter: Sync + Send {
    fn derive_address(&self, index: u32) -> anyhow::Result<String>;
    async fn listen(&self) -> anyhow::Result<()>;
    fn config(&self) -> Arc<ChainConfig>;
}

#[derive(Debug, Clone)]
pub enum Blockchain {
    Evm(EvmBlockchain),
}

impl Blockchain {
    pub fn new(
        config: Arc<ChainConfig>, 
        sender: mpsc::Sender<PaymentEvent>
    ) -> Self {
        match config.chain_type {
            ChainType::EVM => Evm(EvmBlockchain::new(config, sender)),
        }
    }
}

impl BlockchainAdapter for Blockchain {
    fn derive_address(&self, index: u32) -> anyhow::Result<String> {
        match self {
            Evm(bc) => bc.derive_address(index),
        }
    }

    async fn listen(&self) -> anyhow::Result<()> {
        match self {
            Evm(bc) => bc.listen().await,
        }
    }

    fn config(&self) -> Arc<ChainConfig> {
        match self {
            Evm(bc) => bc.config(),
        }
    }
}