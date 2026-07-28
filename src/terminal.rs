#![doc(
    html_logo_url = "https://raw.githubusercontent.com/Daniel-Dos/rust-ollama-chat/master/logo.svg"
)]

//! Aplicação em terminal Rust Rig AI — integração Ollama + busca web MCP + S3.
//!
//! # Fluxo principal
//!
//! 1. Inicializa o `#[tokio::main]`.
//! 2. Sobe o servidor Python MCP via `McpClientManager::init()`.
//! 3. Exibe banner ASCII no terminal.
//! 4. no terminal o usuario informa o prompt.
//! 5. Após o encerra MCP e faz upload da resposta para S3.

mod banner;
mod mcp;
mod rig;
mod aws;

use crate::aws::s3_integration as s3;
use crate::banner::banner_text;
use crate::mcp::web_search_mcp::McpClientManager;
use tracing::log::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|e| {
                warn!("Não foi possivel ler a variavel RUST_LOG, ira seguir no padrao: {e}");
                tracing_subscriber::EnvFilter::new(
                    "warn,terminal=trace,rig_core=trace,reqwest=trace",
                )
            }),
        )
        .init();

    banner_text::print_banner();
    info!(
        "Bem-vindo ao Rust Rig AI! \nIniciando o processo de integração com Ollama e AWS S3...\n"
    );

    let mut mcp_manager = McpClientManager::init().await?;
    info!("MCP client ready");

    println!("Informe o prompt para o teste de integração MCP Tool Calling:");
    let mut prompt_user = String::new();
    std::io::stdin().read_line(&mut prompt_user).expect("Erro na leitura da entrada do usuario.");

    info!("🧪 Executando teste de integração MCP Tool Calling (rig-core v0.40)...");
    let result = match crate::rig::client_ollama::mcp_tool_calling(&mcp_manager, &prompt_user).await {
        Ok(result) => {
            info!("Meu prompt: {}", prompt_user);
            info!("✅ Teste MCP Tool Calling SUCESSO!");
            info!("📊 Tokens: {} total (in: {}, out: {})",
                  result.tokens_total, result.tokens_input, result.tokens_output);
            result.resposta.chars().collect::<String>()
        }
        Err(e) => {
            error!("❌ Falha no teste MCP Tool Calling: {}", e);
            warn!("Ollama e MCP mantidos em execução. Feche o terminal quando quiser.");
            mcp_manager.shutdown().await?;
            return Ok(())
        }
    };

    mcp_manager.shutdown().await?;

    let meu_bucket = s3::get_my_bucket().await?;
    tracing::info!("Obtendo o nome do bucket S3: {}", meu_bucket);
    s3::upload_bucket(&meu_bucket, result).await?;
    Ok(())
}