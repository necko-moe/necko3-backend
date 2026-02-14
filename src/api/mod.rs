pub mod invoice;
pub mod chain;
mod auth;

use crate::config::ChainConfig;
use crate::config::TokenConfig;
use crate::model::ApiResponse;
use crate::model::CreateInvoiceReq;
use crate::model::Empty;
use crate::model::Invoice;
use crate::state::AppState;
use axum::routing::{delete, get, patch, post};
use axum::{middleware, Router};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub use chain::*;
pub use invoice::*;
use crate::api::auth::{auth_middleware, SecurityAddon};

#[derive(OpenApi)]
#[openapi(
    paths(
        add_chain,
        get_chains,
        get_chain,
        delete_chain,
        update_chain,

        add_token,
        get_tokens,
        get_token,
        delete_token,
    
        create_invoice,
        get_invoices,
        get_invoice_by_id,
        delete_invoice
    ),
    components(
        schemas(
            Invoice,
            CreateInvoiceReq,
            ChainConfig,
            TokenConfig,

            ApiResponse<Invoice>,
            ApiResponse<Vec<Invoice>>,
            ApiResponse<ChainConfig>,
            ApiResponse<Vec<ChainConfig>>,
            ApiResponse<TokenConfig>,
            ApiResponse<Vec<TokenConfig>>,
            ApiResponse<String>,
            ApiResponse<Empty>
        )
    ),
    modifiers(&SecurityAddon),
    security(
        ("api_key" = [])
    )
)]
struct ApiDoc;

pub async fn serve(state: Arc<AppState>) -> std::io::Result<()> {
    let app = Router::new()
        .route("/invoice", post(create_invoice))
        .route("/invoice", get(get_invoices))
        .route("/invoice/{id}", get(get_invoice_by_id))
        .route("/invoice/{id}", delete(delete_invoice))

        .route("/chain", post(add_chain))
        .route("/chain", get(get_chains))
        .route("/chain/{name}", get(get_chain))
        .route("/chain/{name}", delete(delete_chain))
        .route("/chain/{name}", patch(update_chain))

        .route("/chain/{name}/token", post(add_token))
        .route("/chain/{name}/token", get(get_tokens))
        .route("/chain/{name}/token/{symbol}", get(get_token))
        .route("/chain/{name}/token/{symbol}", delete(delete_token))

        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))

        .route("/health", get(|| async { "ok" }))
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(state);

    println!("Started listening on http://127.0.0.1:3000");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await
}