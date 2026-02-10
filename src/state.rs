use crate::chain::{Blockchain, BlockchainAdapter};
use crate::config::{ChainConfig, MinChainConfig, TokenConfig};
use crate::model::{Invoice, InvoiceStatus, PaymentEvent};
use alloy::primitives::utils::format_units;
use dashmap::DashMap;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::task::JoinHandle;

pub struct AppState {
    pub tx: Sender<PaymentEvent>,
    pub rx: Mutex<Receiver<PaymentEvent>>,

    pub adapters: RwLock<HashMap<String, Arc<Blockchain>>>,
    pub active_chains: RwLock<HashMap<String, JoinHandle<()>>>,

    pub active_invoices: DashMap<String, Invoice>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, rx): (Sender<PaymentEvent>, Receiver<PaymentEvent>) = mpsc::channel(100);

        Self {
            tx,
            rx: Mutex::new(rx),
            adapters: RwLock::new(HashMap::new()),
            active_chains: RwLock::new(HashMap::new()),
            active_invoices: DashMap::new(),
        }
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
}

impl AppState {
    pub fn start_invoice_watcher(self: Arc<Self>) -> JoinHandle<()> {
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

                    if event.network != invoice.network {
                        println!("bro just sent tokens in network {}, not in {} 💀💀💀",
                                 event.network, invoice.network);
                        continue;
                    }

                    if event.token != invoice.token {
                        println!("bro just sent tokens in {}, not in {} 💀💀💀",
                                 event.token, invoice.token);
                        continue;
                    }

                    let now = chrono::Utc::now();
                    if invoice.expires_at < now {
                        println!("detected transaction on EXPIRED invoice. skipping \
                        (you lost your tokens idiot)");
                        continue
                    }

                    if invoice.status == InvoiceStatus::Paid {
                        println!("(unreachable) invoice is already paid but {} received something",
                                 invoice.address)
                    }

                    invoice.paid_raw += event.amount_raw;

                    let paid = format_units(invoice.paid_raw, event.decimals)
                        .unwrap_or(invoice.paid_raw.to_string());

                    println!("\npaid {}/{} {} on {} (index {})",
                             paid,
                             invoice.amount,
                             invoice.token,
                             invoice.address,
                             invoice.address_index);

                    invoice.paid = paid;

                    let fully_paid = invoice.paid_raw >= invoice.amount_raw;
                    if fully_paid {
                        invoice.status = InvoiceStatus::Paid;
                    }

                    (fully_paid, invoice.id.clone(), invoice.address.clone())
                };

                if is_fully_paid {
                    println!("invoice {} is fully paid!", invoice_id);

                    let maybe_chain_config = {
                        let guard = self.adapters.read().await;
                        guard.get(&event.network).map(|a| a.config())
                    };

                    if let Some(chain_config) = maybe_chain_config {
                        let mut addresses = chain_config
                            .watch_addresses.write().unwrap();
                        addresses.remove(&invoice_address);
                    } else {
                        eprintln!("FATAL: failed to get chain config for {}", event.network);
                    }
                }
            }
        })
    }

    pub fn start_janitor(self: Arc<Self>, interval: Duration) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);

            loop {
                interval.tick().await;
                println!("checking for expired invoices...");

                let now = chrono::Utc::now();
                let mut expired_addresses = Vec::new();

                for entry in self.active_invoices.iter() {
                    let invoice = entry.value();
                    if invoice.status == InvoiceStatus::Pending && invoice.expires_at < now {
                        expired_addresses.push(
                            (invoice.address.clone(), invoice.network.clone(), invoice.id.clone()));
                    }
                }

                for (address, network, invoice_id) in expired_addresses {
                    println!("marking invoice {} (address {}) as expired", invoice_id, address);

                    if let Some(mut inv) = self.active_invoices
                        .get_mut(&address) {
                        inv.status = InvoiceStatus::Expired;
                    }

                    if let Some(chain) = self.adapters.read().await.get(&network) {
                        chain.config().watch_addresses.write().unwrap().remove(&address);
                    }
                }
            }
        })
    }
}

impl AppState {
    pub async fn add_chain(
        &self,
        chain: ChainConfig
    ) -> anyhow::Result<()> {
        let name = chain.name.clone();

        if self.adapters.read().await.contains_key(&name) {
            anyhow::bail!("chain '{}' already exists", name);
        }

        self.adapters.write().await.insert(name.clone(), Arc::new(Blockchain::new(Arc::new(chain),
                                                                                  self.tx.clone())));

        self.start_listening(name).await?;

        Ok(())
    }

    pub async fn get_chains(&self) -> Vec<MinChainConfig> {
        let chains = self.adapters.read().await;

        chains.iter()
            .map(|x| {
                x.1.config().deref().into()
            })
            .collect()
    }

    pub async fn get_chain(
        &self,
        name: &str
    ) -> MinChainConfig {
        let chains = self.adapters.read().await;

        chains.iter()
            .find(|x| x.1.config().name == name)
            .map(|x| {
                x.1.config()
            })
            .unwrap()
            .deref()
            .into()
    }

    pub async fn remove_chain(
        &self,
        name: &str
    ) -> anyhow::Result<()> {
        self.stop_listening(name.to_string()).await?;

        self.adapters.write().await.remove(name);

        Ok(())
    }

    pub async fn add_token(
        &self,
        chain_name: String,
        token: TokenConfig
    ) -> anyhow::Result<()> {
        let guard = self.adapters.read().await;

        let maybe_adapter = guard.get(&chain_name);

        let Some(adapter) = maybe_adapter else {
            anyhow::bail!("chain '{}' does not exist", chain_name);
        };

        adapter.config().tokens.write().unwrap().insert(token);

        Ok(())
    }

    pub async fn remove_token(
        &self,
        chain_name: String,
        symbol: String,
    ) -> anyhow::Result<()> {
        let guard = self.adapters.read().await;

        let maybe_adapter = guard.get(&chain_name);

        let Some(adapter) = maybe_adapter else {
            anyhow::bail!("chain '{}' does not exist", chain_name);
        };

        let chain_config = adapter.config();

        // maybe_TokenConfig
        let mb_tc = chain_config.tokens.read().unwrap().iter()
            .find(|x| x.symbol == symbol)
            .cloned();

        let Some(token_config) = mb_tc else {
            anyhow::bail!("token with symbol '{}' does not exist", symbol);
        };

        chain_config.tokens.write().unwrap().remove(&token_config);

        Ok(())
    }

    pub async fn get_tokens(
        &self,
        chain_name: String
    ) -> anyhow::Result<Vec<TokenConfig>> {
        let guard = self.adapters.read().await;

        let Some(chain) = guard.get(&chain_name) else {
            anyhow::bail!("chain '{}' does not exist", chain_name);
        };

        Ok(chain.config().tokens.read().unwrap().iter().cloned().collect())
    }

    pub async fn get_token(
        &self,
        chain_name: String,
        symbol: String,
    ) -> anyhow::Result<TokenConfig> {
        let guard = self.adapters.read().await;

        let Some(chain) = guard.get(&chain_name) else {
            anyhow::bail!("chain '{}' does not exist", chain_name);
        };

        let mb_tc = chain.config().tokens.read().unwrap().iter()
            .find(|x| x.symbol == symbol)
            .cloned();

        let Some(token_config) = mb_tc else {
            anyhow::bail!("token with symbol '{}' does not exist", symbol);
        };

        Ok(token_config)
    }

    pub async fn start_listening(&self, chain: String) -> anyhow::Result<()> {
        if self.active_chains.read().await.contains_key(&chain) {
            anyhow::bail!("chain {} is already listening", chain);
        }

        let maybe_adapter = { self.adapters.read().await.get(&chain).cloned() };
        let adapter = match maybe_adapter {
            Some(a) => a,
            None => {
                anyhow::bail!("chain {} not found", chain);
            }
        };

        let chain_config = adapter.config();

        let listener = tokio::spawn(async move {
            let chain_name = chain_config.name.clone();
            if let Err(e) = adapter.listen().await {
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