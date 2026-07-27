//! Rotas da API REST — configura os endpoints do servidor Axum.
//!
//! # Endpoints
//!
//! - `GET /api/prompt?texto=<consulta>` — envia um prompt ao modelo Ollama
//!   com tool calling MCP e retorna a resposta gerada.

use axum::{Json, Router};
use axum::extract::State;
use axum::routing::get;
use crate::rest::message::Message;
use crate::rest::prompt_handler;
use crate::rest::state::AppState;

/// Constrói o roteador Axum com as rotas da API.
///
/// Recebe o [`AppState`] com o `McpClientManager` compartilhado e
/// registra a rota `GET /api/prompt` ligada ao handler de prompt.
pub async fn router_api(state: AppState) -> Router {
    Router::new()
        .route("/api/prompt", get(prompt_handler::create_prompt))
        .with_state(state)
}