mod config;
mod model;
mod chain;
mod state;
mod api;

use crate::state::AppState;
use axum::routing::{delete, get, post};
use axum::Router;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = get_app_state();

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))

        .route("/invoice", post(api::create_invoice))
        .route("/invoice", get(api::get_invoices))
        .route("/invoice/{id}", get(api::get_invoice_by_id))
        .route("/invoice/{id}", delete(api::delete_invoice))

        .route("/chain", post(api::add_chain))
        .route("/chain", get(api::get_chains))
        .route("/chain/{name}", get(api::get_chain))
        .route("/chain/{name}", delete(api::remove_chain))

        .route("/chain/{name}/token", post(api::chain::add_token))
        .route("/chain/{name}/token", get(api::chain::get_tokens))
        .route("/chain/{name}/token/{symbol}", get(api::chain::get_token))
        .route("/chain/{name}/token/{symbol}", delete(api::chain::remove_token))
        .with_state(state);

    println!("Started listening on http://127.0.0.1:3000");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn get_app_state() -> Arc<AppState> {
    let state = Arc::new(AppState::new());

    let watcher_state = state.clone();
    watcher_state.start_invoice_watcher();

    let janitor_state = state.clone();
    janitor_state.start_janitor(Duration::from_secs(30));
    
    state
}