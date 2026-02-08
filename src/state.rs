use crate::chain;
use crate::config::{ChainConfig, TokenConfig};
use crate::model::{Invoice, InvoiceStatus, PaymentEvent};
use alloy::primitives::utils::format_units;
use alloy::primitives::Address;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::task::JoinHandle;

pub struct AppState {
    pub tx: Sender<PaymentEvent>,
    pub rx: Mutex<Receiver<PaymentEvent>>,

    // key = chain_config.name; value = chain_config
    pub chains: RwLock<HashMap<String, Arc<ChainConfig>>>,
    pub active_chains: RwLock<HashMap<String, JoinHandle<()>>>,

    pub active_invoices: DashMap<Address, Invoice>,
}

impl AppState {
    pub fn with_chains(chains: Vec<ChainConfig>) -> Self {
        let mut chains_map = HashMap::new();
        chains.into_iter().for_each(|chain| {
            chains_map.insert(chain.name.clone(), Arc::new(chain)); });

        let (tx, rx): (Sender<PaymentEvent>, Receiver<PaymentEvent>) = mpsc::channel(100);

        Self {
            tx,
            rx: Mutex::new(rx),
            chains: RwLock::new(chains_map),
            active_chains: RwLock::new(HashMap::new()),
            active_invoices: DashMap::new(),
        }
    }

    pub fn new() -> Self {
        Self::with_chains(vec![])
    }

    pub fn get_free_slot(&self) -> Option<u32> {
        let busy_indexes: Vec<u32> = self.active_invoices.iter()
            .filter(|i|
                i.value().status != InvoiceStatus::Paid)
            .map(|i| i.value().address_index)
            .collect();

        for i in 0..=busy_indexes.len() {
            if !busy_indexes.contains(&(i as u32)) { return Some(i as u32); }
        }

        None // actually unreachable, but who knows
    }

    pub fn start_invoice_watcher(self: Arc<Self>) -> () {
        tokio::spawn(async move {
            let state_clone = self.clone();
            let mut rx = state_clone.rx.lock().await;

            while let Some(event) = rx.recv().await {
                let (is_fully_paid, invoice_id, invoice_address) = {
                    let Some(mut invoice) = self.active_invoices
                        .get_mut(&event.to) else {
                        eprintln!("Unable to find invoice for {}", event.to);
                        continue;
                    };

                    if event.token != invoice.token {
                        println!("bro just sent tokens in {}, not in {} 💀💀💀",
                                 event.token, invoice.token);
                        continue;
                    }

                    let amount_human = format_units(invoice.paid, event.decimals)
                        .unwrap_or(invoice.paid.to_string());

                    if invoice.status == InvoiceStatus::Paid {
                        println!("(unreachable) invoice is already paid but {} received {} {}",
                                 invoice.address, amount_human, invoice.token)
                    }

                    invoice.paid += event.amount_raw;

                    println!("\npaid {}/{} on {} (index {})",
                             amount_human,
                             invoice.amount,
                             invoice.address,
                             invoice.address_index);

                    let fully_paid = invoice.paid >= invoice.amount_raw;
                    if fully_paid {
                        invoice.status = InvoiceStatus::Paid;
                    }

                    (fully_paid, invoice.id.clone(), invoice.address)
                };

                if is_fully_paid {
                    println!("invoice {} is fully paid!", invoice_id);

                    let maybe_chain_config = {
                        let guard = self.chains.read().await;
                        guard.get(&event.network).cloned()
                    };

                    if let Some(chain_config) = maybe_chain_config {
                        let mut addresses = chain_config
                            .watch_addresses.write().await;
                        addresses.retain(|a| *a != invoice_address);
                    } else {
                        eprintln!("FATAL: failed to get chain config for {}", event.network);
                    }
                }
            }
        });
    }
}

impl AppState {
    pub async fn add_chain(
        &mut self,
        chain: ChainConfig
    ) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn add_token(
        &mut self,
        chain_name: String,
        token: TokenConfig
    ) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn start_listening(&self, chain: String) -> anyhow::Result<()> {
        if self.active_chains.read().await.contains_key(&chain) {
            anyhow::bail!("chain {} is already listening", chain);
        }

        let maybe_config = { self.chains.read().await.get(&chain).cloned() };
        let chain_config = match maybe_config {
            Some(cc) => cc,
            None => {
                anyhow::bail!("chain {} not found", chain);
            }
        };

        let tx = self.tx.clone();
        let listener = tokio::spawn(async move {
            let chain_name = chain_config.name.clone();
            if let Err(e) = chain::listen_on(chain_config, tx).await {
                eprintln!("{} listener died: {}", chain_name, e);
            }
        });

        self.active_chains.write().await.insert(chain.clone(), listener);

        Ok(())
    }

    pub async fn stop_listening(&self, chain: String) -> anyhow::Result<()> {
        let mut active_chains = self.active_chains.write().await;

        if let Some(handle) = active_chains.remove(&chain) {
            handle.abort();
        } else {
            anyhow::bail!("chain {} is not listening", chain);
        }

        Ok(())
    }
}