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
use std::sync::Arc;
use std::time::Duration;
use coins_bip32::prelude::{Parent, XPub};
use tokio::sync::mpsc;
use url::Url;

const LAG: u64 = 3;

sol! {
    #[derive(Debug)]
    event Transfer(address indexed from, address indexed to, uint256 value);
}

pub fn get_address(xpub: XPub, index: u32) -> anyhow::Result<Address> {
    let child_xpub = xpub.derive_child(index)?;
    let verifying_key = child_xpub.as_ref();

    Ok(Address::from_public_key(&verifying_key))
}

pub async fn listen_on(
    config: Arc<ChainConfig>,
    sender: mpsc::Sender<PaymentEvent>
) -> anyhow::Result<()> {
    let rpc_url = Url::parse(&config.rpc_url)?;
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
                    let guard = config.watch_addresses.read().await;
                    guard.clone()
                };

                let transactions = process_block(&addresses, block)
                    .unwrap_or_default();
                for tx in transactions {
                    let amount_human = format_units(tx.value(), config.decimals)?;

                    let event = PaymentEvent {
                        network: config.name.clone(),
                        tx_hash: tx.tx_hash(),
                        from: tx.from(),
                        to: tx.to().unwrap_or_default(), // default is unreachable, but it's better
                                                         // to keep this instead of ::unwrap()
                        token: config.native_symbol.clone(),
                        amount: amount_human,
                        amount_raw: tx.value(),
                        decimals: config.decimals,
                    };

                    let _ = sender.send(event).await;
                }
            }

            let config = config.clone();
            let sender = sender.clone();
            process_logs(config, block_num, &provider, sender).await;
        }

        last_block_num = current_block_num;
    }
}

fn process_block(
    addresses: &[Address],
    block: Block,
) -> anyhow::Result<Vec<RpcTransaction>> {
    let txs = block.into_transactions_vec();

    let transactions = txs
        .into_iter()
        .filter(|tx| {
            tx.to().map_or(false, |to|
                addresses.contains(&to))
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
    let token_addresses: Vec<Address> = config.tokens.iter()
        .map(|t| t.contract)
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
                let guard = config.watch_addresses.read().await;
                guard.clone()
            };

            if addresses.contains(&event_data.to) {
                let maybe_conf = config.tokens.iter()
                    .find(|t| t.contract == event_data.address);
                //    .unwrap(); // trust me bro :)

                let Some(token_conf) = maybe_conf else {
                    eprintln!("(should be unreachable) received log from UNKNOWN contract: {}", event_data.address);
                    // NEVER trust anyone
                    return;
                };

                let amount_human = format_units(event_data.value, token_conf.decimals)
                    .unwrap_or_default();

                let event = PaymentEvent {
                    network: config.name.clone(),
                    tx_hash: log.transaction_hash.unwrap_or_default(),
                    from: event_data.from,
                    to: event_data.to,
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