#![doc(
    html_logo_url = "https://raw.githubusercontent.com/Daniel-Dos/rust-ollama-chat/master/logo.svg"
)]

//! Aplicação desktop Rust Rig AI — integração Ollama + busca web MCP + S3.
//!
//! # Fluxo principal
//!
//! 1. Inicializa `tokio::runtime` (sem `#[tokio::main]` para controle manual).
//! 2. Sobe o servidor Python MCP via `McpClientManager::init()`.
//! 3. Exibe banner ASCII no terminal.
//! 4. Abre janela Iced (splash 5s → chat com toggle de busca web).
//! 5. Após fechar a janela, encerra MCP e faz upload da resposta para S3.

use crate::aws::s3_integration as s3;
use crate::banner::banner_text;
use crate::mcp::web_search_mcp::McpClientManager;
use tracing::{error, info, warn};

mod aws;
mod banner;
mod gui;
mod mcp;
mod rig;

fn main() -> Result<(), anyhow::Error> {
    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|e| {
                warn!("Não foi possivel ler a variavel RUST_LOG, ira seguir no padrao: {e}");
                tracing_subscriber::EnvFilter::new(
                    "warn,main=trace,rig_core=trace,reqwest=trace",
                )
            }),
        )
        .init();

    banner_text::print_banner();
    info!(
        "Bem-vindo ao Rust Rig AI! \nIniciando o processo de integração com Ollama e AWS S3...\n"
    );

    let mut mcp_manager = rt.block_on(McpClientManager::init())?;
    info!("MCP client ready");

    info!("Iniciando interface gráfica...");
    std::thread::sleep(std::time::Duration::from_secs(3));

    let resposta = match gui::app::run(&mcp_manager) {
        Ok(r) => r,
        Err(e) => {
            error!("{e}");
            warn!("Ollama e MCP mantidos em execução. Feche o terminal quando quiser.");
            rt.block_on(mcp_manager.shutdown())?;
            return Ok(());
        }
    };

    rt.block_on(mcp_manager.shutdown())?;

    let meu_bucket = rt.block_on(s3::get_my_bucket())?;
    info!("Obtendo o nome do bucket S3: {}", meu_bucket);
    rt.block_on(s3::upload_bucket(&meu_bucket, resposta))?;
    Ok(())
}
