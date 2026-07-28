//! Servidor REST para o Rust Rig AI — expõe o modelo Ollama via API HTTP.
//!
//! # Fluxo
//!
//! 1. Inicializa o servidor Python MCP via `McpClientManager::init()`.
//! 2. Exibe banner ASCII no terminal.
//! 3. Inicia servidor Axum em `http://localhost:8080`.
//! 4. Aguarda requisições `GET /api/prompt?texto=<consulta>`.
//! 5. No encerramento (Ctrl+C), faz shutdown do MCP.

mod rest;
mod banner;
mod mcp;
mod rig;
mod aws;

use std::sync::Arc;
use tracing::log::{error, info, warn};
use crate::banner::banner_text;
use crate::mcp::web_search_mcp::McpClientManager;
use crate::rest::router_api as rest_router;
use crate::rest::server_app as rest_server;
use crate::rest::state::AppState;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|e| {
                warn!("Não foi possivel ler a variavel RUST_LOG, ira seguir no padrao: {e}");
                tracing_subscriber::EnvFilter::new(
                    "warn,api_rest=trace,rig_core=trace,reqwest=trace",
                )
            }),
        )
        .init();

    banner_text::print_banner();
    info!(
        "Bem-vindo ao Rust Rig AI! \nIniciando o processo de integração com Ollama e AWS S3...\n"
    );

    let mcp_manager = McpClientManager::init().await?;
    info!("MCP client ready");

    let arc_mcp = Arc::new(mcp_manager);
    let state = AppState {
        mcp_manager: Arc::clone(&arc_mcp),
    };

    let app = rest_router::router_api(state).await;
    rest_server::server(app).await?;

    let mut mcp_manager = Arc::into_inner(arc_mcp)
        .expect("mcp_manager should have no other references after server shutdown");
    if let Err(e) = mcp_manager.shutdown().await {
        error!("Erro ao encerrar MCP: {e}");
    }
    Ok(())
}
