use std::fmt::{Display, Formatter};
use crate::config::ChainConfig;
use crate::model::PaymentEvent;
use std::sync::Arc;
use alloy::primitives::Address;
use coins_bip32::prelude::XPub;
use tokio::sync::mpsc;

pub mod evm;

#[derive(Debug, Copy, Clone)]
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

pub fn listen_on(
    config: Arc<ChainConfig>,
    sender: mpsc::Sender<PaymentEvent>,
) -> impl Future<Output = anyhow::Result<()>> {
    match config.chain_type {
        ChainType::EVM => evm::listen_on(config, sender),
    }
}

pub fn get_address(
    chain_type: ChainType,
    xpub: XPub,
    index: u32
) -> anyhow::Result<Address> {
    match chain_type {
        ChainType::EVM => evm::get_address(xpub, index),
    }
}