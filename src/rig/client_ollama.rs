//! Chat com modelos Ollama — com e sem busca web.
//!
//! Fornece duas funções principais de chat:
//! - [`resposta_chat_peer`]: faz busca web via MCP antes de consultar o modelo.
//! - [`chat_direct`]: envia o prompt diretamente ao modelo, sem busca web.
//!
//! Ambas retornam [`ChatResult`] com a resposta textual e métricas de tokens
//! obtidas via [`extended_details`](https://docs.rs/rig-core/latest/rig_core/agent/prompt_request/struct.PromptRequest.html#method.extended_details).

use crate::rig::web_search;
use rmcp::{Peer, RoleClient};
use rig_core::agent::{Agent, PromptResponse};
use rig_core::client::{CompletionClient, Nothing};
use rig_core::completion::Prompt;
use rig_core::providers::ollama;
use rig_core::providers::ollama::OllamaExt;
use tracing::info;

/// Resultado de uma chamada de chat, contendo a resposta e métricas de tokens.
#[derive(Debug, Clone)]
pub struct ChatResult {
    /// Texto da resposta gerada pelo modelo.
    pub resposta: String,
    /// Quantidade de tokens de entrada (prompt) consumidos.
    pub tokens_input: u64,
    /// Quantidade de tokens de saída (completion) gerados.
    pub tokens_output: u64,
    /// Total de tokens consumidos na requisição (entrada + saída).
    pub tokens_total: u64,
}

/// Envia um prompt ao modelo com contexto de busca web via MCP.
///
/// # Funcionamento
///
/// 1. Extrai URLs do prompt com [`web_search::extrair_url`].
/// 2. Se encontrar URL, faz fetch do conteúdo com [`web_search::fetch_url`].
/// 3. Caso contrário, faz busca com [`web_search::search_and_fetch`].
/// 4. Monta um prompt final combinando pergunta + resultados da web.
/// 5. Envia ao modelo Ollama e retorna a resposta com métricas de tokens.
///
/// # Errors
///
/// Retorna erro se a variável `OLLAMA_MODEL` não estiver definida, se a
/// comunicação com o servidor MCP falhar, ou se o Ollama retornar erro.
pub async fn resposta_chat_peer(
    peer: Peer<RoleClient>,
    prompt: String,
) -> Result<ChatResult, anyhow::Error> {
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

    let agente: AgentMCP = rust_agente().await?;
    let pendente = agente.prompt(prompt_final);
    let resp: PromptResponse = pendente.extended_details().await?;
    info!("{}", resp.output);
    Ok(ChatResult {
        tokens_input: resp.usage.input_tokens,
        tokens_output: resp.usage.output_tokens,
        tokens_total: resp.usage.total_tokens,
        resposta: resp.output,
    })
}

/// Envia um prompt diretamente ao modelo Ollama, sem busca web.
///
/// Útil para perguntas que não dependem de informações externas ou
/// quando se deseja uma resposta mais rápida sem overhead de rede.
///
/// # Errors
///
/// Retorna erro se a variável `OLLAMA_MODEL` não estiver definida
/// ou se o Ollama retornar erro.
pub async fn chat_direct(prompt: String) -> Result<ChatResult, anyhow::Error> {
    let agente: AgentMCP = rust_agente().await?;
    let pendente = agente.prompt(&prompt);
    let resp: PromptResponse = pendente.extended_details().await?;
    info!("{}", resp.output);
    Ok(ChatResult {
        tokens_input: resp.usage.input_tokens,
        tokens_output: resp.usage.output_tokens,
        tokens_total: resp.usage.total_tokens,
        resposta: resp.output,
    })
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
