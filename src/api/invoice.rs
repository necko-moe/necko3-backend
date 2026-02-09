use crate::chain;
use crate::model::{CreateInvoiceReq, Invoice, InvoiceStatus};
use crate::state::AppState;
use alloy::primitives::utils::parse_units;
use alloy::primitives::U256;
use axum::extract::{Path, State};
use axum::Json;
use coins_bip32::prelude::XPub;
use std::str::FromStr;
use std::sync::Arc;

pub async fn create_invoice(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateInvoiceReq>,
) -> String {
    let chain_config = {
        let guard = state.chains.read().await;
        let maybe_config = guard.get(&payload.network);

        let Some(cc) = maybe_config.cloned() else {
            return format!("Error: network '{}' is not currently supported", payload.network);
        };

        cc
    };

    let token_decimals = {
        let guard = chain_config.tokens.read().unwrap();

        let token_conf = guard.iter()
            .find(|t| t.symbol == payload.token);
        if token_conf.is_none() && payload.token != chain_config.native_symbol {
            return format!("Error: token '{}' is not currently supported on {}",
                           payload.token, payload.network);
        };

        token_conf
            .map(|t| t.decimals)
            .unwrap_or(chain_config.decimals)
    };

    let amount_raw = match parse_units(&payload.amount, token_decimals) {
        Ok(a) => a,
        Err(e) => {
            return format!("Error while trying to parse units: {}", e)
        }
    };

    let Some(index) = state.get_free_slot() else {
        return "Error: no free slots available".into();
    };

    // todo: this is actually hardcoded EVM shit (fix this)
    let xpub_str = "xpub6EeaXhbbgvtV6KF1fvBeEn7DZnd1Gd4xh36eMAAeBB4KA73ZV5pXmjyddjPziE5QqkcoH\
    tRRpkce9UP5qxsd2Q9qi3zmeXtEz5sc7NFGcvN";
    let xpub = XPub::from_str(xpub_str)
        .expect("Invalid Xpub string");
    // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

    let address = match chain::get_address(chain_config.chain_type, xpub, index) {
        Ok(a) => a,
        Err(e) => {
            return format!("Error: failed to get address (index {}) for {} chain: {}",
                           index, chain_config.chain_type, e);
        }
    };

    let invoice = Invoice {
        id: uuid::Uuid::new_v4().to_string(),
        address_index: index,
        address,
        amount: payload.amount,
        amount_raw: amount_raw.into(),
        paid: U256::from(0),
        token: payload.token,
        network: payload.network,
        created_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::minutes(15),
        status: InvoiceStatus::Pending,
    };

    state.active_invoices.insert(address, invoice);
    chain_config.watch_addresses.write().unwrap().insert(address.clone());

    format!("Pay to: {:?} (index {})", address, index)
}

pub async fn get_invoices(
    State(state): State<Arc<AppState>>
) -> Json<Vec<Invoice>> {
    Json(state.active_invoices.iter()
        .map(|x| x.value().clone())
        .collect::<Vec<_>>())
}

pub async fn get_invoice_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Invoice>, String> {
    let invoice = state.active_invoices.iter()
        .find(|x| x.id == id)
        .map(|x| x.value().clone());

    match invoice {
        Some(inv) => Ok(Json(inv)),
        None => Err("invoice not found".to_owned()),
    }
}

pub async fn delete_invoice(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> String {
    let maybe_address = state.active_invoices.iter()
        .find(|x| x.id == id)
        .map(|x| x.key().clone());

    if let Some(address) = maybe_address {
        if state.active_invoices.remove(&address).is_some() {
            return "ok".to_owned();
        }
    }

    "invoice not found".to_owned()
}