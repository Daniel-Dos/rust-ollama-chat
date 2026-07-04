use std::io;

use crate::aws::s3_integration as s3;
use crate::banner::banner_text;
use crate::mcp::web_search_mcp::McpClientManager;
use crate::rig::client_ollama as rust_ollama;
use tracing::info;

#[allow(dead_code)]
mod aws;
mod banner;
mod mcp;
mod rig;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn,Rust_Rig_AI=info")),
        )
        .init();

    banner_text::print_banner();
    println!("Bem-vindo ao Rust Rig AI! \nIniciando o processo de integração com Ollama e AWS S3...\n");

    let mut mcp_manager = McpClientManager::init().await?;
    info!("MCP client ready");

    println!("Informe o Prompt para o modelo Ollama (em Português):");
    let mut prompt = String::new();
    io::stdin()
        .read_line(&mut prompt)
        .expect("Erro ao ler a entrada do prompt.");

    let resposta = rust_ollama::resposta_chat(&mcp_manager, prompt).await?;

    mcp_manager.shutdown().await?;

    let meu_bucket = s3::get_my_bucket().await?;
    info!("Obtendo o nome do bucket S3: {}", meu_bucket);
    s3::upload_bucket(meu_bucket, resposta).await?;
    Ok(())
}
