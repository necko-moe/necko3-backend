use crate::chain::BlockchainAdapter;
use crate::model::{CreateInvoiceReq, Invoice, InvoiceStatus};
use crate::state::AppState;
use alloy::primitives::utils::parse_units;
use alloy::primitives::U256;
use axum::extract::{Path, State};
use axum::Json;
use std::ops::Deref;
use std::sync::Arc;

pub async fn create_invoice(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateInvoiceReq>,
) -> String {
    let adapter = {
        let guard = state.adapters.read().await;
        let maybe_adapter = guard.get(&payload.network);

        let Some(cc) = maybe_adapter else {
            return format!("Error: network '{}' is not currently supported", payload.network);
        };

        cc.deref().clone()
    };
    
    let chain_config = adapter.config();

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

    let address = match adapter.derive_address(index) {
        Ok(a) => a,
        Err(e) => {
            return format!("Error: failed to get address (index {}) for {} chain: {}",
                           index, chain_config.chain_type, e);
        }
    };

    let invoice = Invoice {
        id: uuid::Uuid::new_v4().to_string(),
        address_index: index,
        address: address.clone(),
        amount: payload.amount,
        amount_raw: amount_raw.into(),
        paid: "0".to_string(),
        paid_raw: U256::from(0),
        token: payload.token,
        network: payload.network,
        created_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::minutes(15),
        status: InvoiceStatus::Pending,
    };

    state.active_invoices.insert(address.clone(), invoice);
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