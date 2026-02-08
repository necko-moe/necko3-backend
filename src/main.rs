mod config;
mod model;
mod chain;
mod state;
mod api;

use crate::chain::ChainType;
use crate::config::{ChainConfig, TokenConfig};
use crate::state::AppState;
use alloy::primitives::address;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let usdc_polygon = TokenConfig {
        symbol: "USDC".to_string(),
        contract: address!("0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359"),
        decimals: 6,
    };

    let polygon_conf = ChainConfig {
        name: "Polygon".to_owned(),
        rpc_url: "https://polygon-bor-rpc.publicnode.com".to_owned(),
        chain_type: ChainType::EVM,
        native_symbol: "POL".to_owned(),
        decimals: 18,
        watch_addresses: RwLock::new(vec![]),
        tokens: vec![usdc_polygon]
    };

    let state = Arc::new(AppState::with_chains(vec![
        polygon_conf
    ]));
    state.start_listening("Polygon".to_owned()).await?;

    let watcher_state = state.clone();
    watcher_state.start_invoice_watcher();

    let app = Router::new()
        .route("/invoice/create", post(api::create_invoice))
        .route("/invoice/list", get(api::get_invoices))
        .route("/health", get(|| async { "ok" }))
        .with_state(state);

    println!("Started listening on http://127.0.0.1:3000");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
