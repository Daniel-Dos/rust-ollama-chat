//! Handler do endpoint `GET /api/prompt` — integração com Ollama + MCP.

use crate::rest::message::Message;
use crate::rest::state::AppState;
use crate::rig::client_ollama as ollama;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;

/// Processa uma requisição de prompt recebida via `GET /api/prompt`.
///
/// Extrai o texto da query string, chama o modelo Ollama com tool calling
/// MCP nativo via [`ollama::mcp_tool_calling`], e retorna a resposta
/// gerada como JSON.
///
/// # Errors
///
/// Retorna `500 Internal Server Error` se a chamada ao modelo falhar.
pub async fn create_prompt(
    State(state): State<AppState>,
    Query(mut message): Query<Message>,
) -> Result<Json<Message>, (StatusCode, String)> {
    let result = ollama::mcp_tool_calling(state.mcp_manager.as_ref(), &message.texto)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    message.texto = result.resposta;
    Ok(Json(message))
}
