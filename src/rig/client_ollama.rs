use crate::mcp::web_search_mcp::McpClientManager;
use crate::rig::web_search;
use rig_core::agent::Agent;
use rig_core::client::{CompletionClient, Nothing};
use rig_core::completion::Prompt;
use rig_core::providers::ollama;
use rig_core::providers::ollama::OllamaExt;
use tracing::info;

pub async fn resposta_chat(
    mcp: &McpClientManager,
    prompt: String,
) -> Result<String, anyhow::Error> {
    let peer = mcp.peer();
    let urls = web_search::extrair_url(&prompt);

    let contexto = if let Some(url) = urls.first() {
        info!("URL detectada — web_fetch: {url}");
        web_search::fetch_url(&peer, url).await?
    } else {
        info!("Nenhuma URL — web_search + fetch");
        web_search::search_and_fetch(&peer, &prompt).await?
    };

    let prompt_final = format!(
        "Pergunta: {}\n\n\
         Resultados da web:\n{}\n\n\
         Com base APENAS nos resultados acima, responda em português \
         de forma clara e completa.",
        prompt, contexto
    );

    let resposta = rust_agente().await?.prompt(prompt_final).await?;
    info!("{}", resposta);
    Ok(resposta)
}

fn ollama_client() -> Result<rig_core::client::Client<OllamaExt>, anyhow::Error> {
    ollama::Client::new(Nothing).map_err(|e| anyhow::anyhow!("Failed to create Ollama client: {e}"))
}

type AgentMCP = Agent<rig_core::providers::ollama::CompletionModel>;

async fn rust_agente() -> Result<AgentMCP, anyhow::Error> {
    let model = std::env::var("OLLAMA_MODEL")
        .map_err(|e| anyhow::anyhow!("Variável de ambiente OLLAMA_MODEL não definida: {e}"))?;
    info!("Client Ollama - model: {}", model);

    let client = ollama_client()?;

    let rust_agente = client
        .agent(model)
        .preamble("Você é um assistente de IA útil, amigável e prestativo. Responda às perguntas de forma clara e concisa. Responda sempre em português (Brasil).")
        .build();

    Ok(rust_agente)
}
