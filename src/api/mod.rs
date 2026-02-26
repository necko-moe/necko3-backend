mod invoice;
mod chain;
mod auth;
mod payment;
mod webhook;
mod public;

use crate::model::{CreateInvoiceReq};
use crate::model::core::{InvoiceSchema, ChainConfigSchema, TokenConfigSchema, WebhookSchema,
                         PaymentSchema};
use necko3_core::state::AppState;
use axum::routing::{delete, get, patch, post};
use axum::{middleware, Router};
use std::sync::Arc;
use axum::http::{header, HeaderName, HeaderValue, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub use chain::*;
pub use invoice::*;
pub use payment::*;
pub use webhook::*;
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
        cancel_invoice,

        get_payment,
        get_payments,
        cancel_payment,

        get_webhook,
        get_webhooks,
        cancel_webhook,

        public::get_invoice_data,
        public::get_invoice_payments
    ),
    components(
        schemas(
            InvoiceSchema,
            CreateInvoiceReq,
            ChainConfigSchema,
            TokenConfigSchema,
            WebhookSchema,
            PaymentSchema
        )
    ),
    modifiers(&SecurityAddon),
    security(
        ("api_key" = [])
    )
)]
struct ApiDoc;

pub async fn serve(
    state: Arc<AppState>,
    include_swagger: bool,
    cors_layer: CorsLayer,
    bind_address: &str,
) -> std::io::Result<()> {
    let mut app = Router::new()
        .route("/invoice", post(create_invoice))
        .route("/invoice", get(get_invoices))
        .route("/invoice/{id}", get(get_invoice_by_id))
        .route("/invoice/{id}", delete(cancel_invoice))

        .route("/chain", post(add_chain))
        .route("/chain", get(get_chains))
        .route("/chain/{name}", get(get_chain))
        .route("/chain/{name}", delete(delete_chain))
        .route("/chain/{name}", patch(update_chain))

        .route("/chain/{name}/token", post(add_token))
        .route("/chain/{name}/token", get(get_tokens))
        .route("/chain/{name}/token/{symbol}", get(get_token))
        .route("/chain/{name}/token/{symbol}", delete(delete_token))

        .route("/payment", get(get_payments))
        .route("/payment/{id}", get(get_payment))
        .route("/payment/{id}", delete(cancel_payment))

        .route("/webhook", get(get_webhooks))
        .route("/webhook/{id}", get(get_webhook))
        .route("/webhook/{id}", delete(cancel_webhook))

        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))

        .route("/public/invoice/{id}", get(public::get_invoice_data))
        .route("/public/invoice/{id}/payments", get(public::get_invoice_payments))

        .layer(cors_layer)
        .layer(TraceLayer::new_for_http())

        .route("/health", get(|| async { "ok" }));


    if include_swagger {
        info!("Swagger UI enabled at /swagger-ui");
        app = app.merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", ApiDoc::openapi()))
    }

    info!("Starting server on http://{}", bind_address);

    let listener = tokio::net::TcpListener::bind(bind_address).await.map_err(|e| {
        error!(address = %bind_address, error = %e, "Failed to bind TCP listener");
        e
    })?;
    
    axum::serve(listener, app.with_state(state)).await
}

pub fn cors_from_str(raw_str: &str) -> CorsLayer {
    let (origin, allow_credentials) = match raw_str.to_lowercase().as_str() {
        "all" | "any" => (AllowOrigin::any(), false),
        _ => {
            let list = raw_str
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<HeaderValue>()
                    .expect("Bad origin"))
                .collect::<Vec<HeaderValue>>();

            (AllowOrigin::list(list), true)
        }
    };

    CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::PUT, Method::DELETE])
        .allow_headers([
            header::CONTENT_TYPE,
            header::ACCEPT,
            HeaderName::from_static("x-api-key"),
        ])
        .allow_credentials(allow_credentials)
}