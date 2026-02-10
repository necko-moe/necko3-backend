use crate::chain::BlockchainAdapter;
use crate::config::ChainConfig;
use crate::model::PaymentEvent;
use alloy::consensus::Transaction;
use alloy::network::TransactionResponse;
use alloy::primitives::utils::format_units;
use alloy::primitives::{Address, BlockNumber};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::Transaction as RpcTransaction;
use alloy::rpc::types::{Block, Filter};
use alloy::sol;
use coins_bip32::prelude::{Parent, XPub};
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use url::Url;

const LAG: u64 = 3;

sol! {
    #[derive(Debug)]
    event Transfer(address indexed from, address indexed to, uint256 value);
}

#[derive(Debug, Clone)]
pub struct EvmBlockchain {
    config: Arc<ChainConfig>,
    sender: mpsc::Sender<PaymentEvent>,
}

impl EvmBlockchain {
    pub fn new(
        config: Arc<ChainConfig>,
        sender: mpsc::Sender<PaymentEvent>
    ) -> Self {
        Self { config, sender }
    }
}

impl BlockchainAdapter for EvmBlockchain {
    fn derive_address(&self, index: u32) -> anyhow::Result<String> {
        let xpub = XPub::from_str(&self.config.xpub)?;

        let child_xpub = xpub.derive_child(index)?;
        let verifying_key = child_xpub.as_ref();

        Ok(Address::from_public_key(&verifying_key).to_string())
    }

    async fn listen(&self) -> anyhow::Result<()> {
        let rpc_url = Url::parse(&self.config.rpc_url)?;
        let provider = ProviderBuilder::new().connect_http(rpc_url);

        let mut last_block_num = match provider.get_block_number().await {
            Ok(n) => n,
            Err(e) => {
                eprintln!("failed to get latest block number: {}. retrying in 5s...", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                provider.get_block_number().await?
            }
        };

        if last_block_num <= LAG { return Ok(()); } // better to be safe than sorry

        loop {
            let current_block_num = match provider.get_block_number().await {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("failed to get latest block number: {}. sleep 2s...", e);
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    continue
                }
            } - LAG;

            if current_block_num <= last_block_num {
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }

            for block_num in (last_block_num + 1)..=current_block_num {
                println!("processing block {}...", block_num);

                if let Some(block) = provider
                    .get_block_by_number(block_num.into())
                    .full()
                    .await
                    .ok()
                    .flatten()
                {
                    let addresses = {
                        let guard = self.config.watch_addresses.read().unwrap();
                        guard.clone()
                    };

                    let transactions = process_block(&addresses, block)
                        .unwrap_or_default();
                    for tx in transactions {
                        let amount_human = format_units(tx.value(), self.config.decimals)?;

                        let event = PaymentEvent {
                            network: self.config.name.clone(),
                            tx_hash: tx.tx_hash(),
                            from: tx.from().to_string(),
                            to: tx.to().unwrap_or_default().to_string(), // default is unreachable,
                            // but it's better to keep this instead of ::unwrap()
                            token: self.config.native_symbol.clone(),
                            amount: amount_human,
                            amount_raw: tx.value(),
                            decimals: self.config.decimals,
                        };

                        let _ = self.sender.send(event).await;
                    }
                }

                let config = self.config.clone();
                let sender = self.sender.clone();
                process_logs(config, block_num, &provider, sender).await;
            }

            last_block_num = current_block_num;
        }
    }

    fn config(&self) -> Arc<ChainConfig> {
        self.config.clone()
    }
}

fn process_block(
    addresses: &HashSet<String>,
    block: Block,
) -> anyhow::Result<Vec<RpcTransaction>> {
    let txs = block.into_transactions_vec();

    let transactions = txs
        .into_iter()
        .filter(|tx| {
            tx.to().map_or(false, |to|
                addresses.contains(&to.to_string()))
        })
        .collect();

    Ok(transactions)
}

async fn process_logs(
    config: Arc<ChainConfig>,
    block_number: BlockNumber,
    provider: &impl Provider,
    sender: mpsc::Sender<PaymentEvent>,
) {
    let token_addresses: Vec<Address> = config.tokens.read().unwrap().iter()
        .map(|t| Address::from_str(&t.contract).unwrap_or_default())
        .collect();

    if token_addresses.is_empty() { return; }

    let filter = Filter::new()
        .from_block(block_number)
        .to_block(block_number)
        .address(token_addresses)
        .event("Transfer(address,address,uint256)");

    let logs = match provider.get_logs(&filter).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("failed to get logs from {}: {}. Retrying in 3s...", config.name, e);
            tokio::time::sleep(Duration::from_secs(3)).await;
            provider.get_logs(&filter).await.unwrap_or_default()
        }
    };

    for log in logs {
        if let Ok(transfer) = log.log_decode::<Transfer>() {
            let event_data = transfer.inner;

            let addresses = {
                let guard = config.watch_addresses.read().unwrap();
                guard.clone()
            };

            if addresses.contains(&event_data.to.to_string()) {
                let token_conf = {
                    let guard = config.tokens.read().unwrap();
                    let maybe_conf = guard.iter()
                        .find(|t| t.contract == event_data.address.to_string());
                        // .unwrap(); // trust me bro :)

                    match maybe_conf.cloned() {
                        Some(tc) => tc,
                        None => {
                            eprintln!("(should be unreachable) received log from UNKNOWN contract \
                            for {}", event_data.address);
                            // NEVER trust anyone
                            return;
                        }
                    }
                };

                let amount_human = format_units(event_data.value, token_conf.decimals)
                    .unwrap_or_default();

                let event = PaymentEvent {
                    network: config.name.clone(),
                    tx_hash: log.transaction_hash.unwrap_or_default(),
                    from: event_data.from.to_string(),
                    to: event_data.to.to_string(),
                    token: token_conf.symbol.clone(),
                    amount: amount_human,
                    amount_raw: event_data.value,
                    decimals: token_conf.decimals,
                };

                let _ = sender.send(event).await;
            }
        }
    }
}