pub mod invoice;
pub mod chain;
pub mod auth;

use crate::model::ApiResponse;
use crate::model::CreateInvoiceReq;
use crate::model::Empty;
use necko3_core::model::{Invoice, ChainConfig, TokenConfig};
use necko3_core::state::AppState;
use axum::routing::{delete, get, patch, post};
use axum::{middleware, Router};
use std::sync::Arc;
use alloy::transports::http::reqwest::header::HeaderName;
use axum::http::{header, HeaderValue, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};
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

pub async fn serve(
    state: Arc<AppState>,
    include_swagger: bool,
    cors_layer: CorsLayer
) -> std::io::Result<()> {
    let mut app = Router::new()
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
        .layer(cors_layer)

        .route("/health", get(|| async { "ok" }));


    if include_swagger {
        app = app.merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", ApiDoc::openapi()))
    }

    println!("Started listening on http://127.0.0.1:3000");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
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