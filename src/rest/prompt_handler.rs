//! Handler do endpoint `GET /api/prompt` — integração com Ollama + MCP.

use axum::body::HttpBody;
use crate::rest::message::Message;
use crate::rest::state::AppState;
use crate::rig::client_ollama as ollama;
use crate::aws::s3_integration as s3;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use tracing::info;
use crate::rest::message_response::MessageResponse;

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
    mut message: Json<Message>,
) -> Result<Json<MessageResponse>, (StatusCode, String)> {
    let result = ollama::mcp_tool_calling(state.mcp_manager.as_ref(), &message.prompt)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    message.prompt = result.resposta;

    let mybucker = s3::get_my_bucket().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let file_name_up = s3::upload_bucket(&mybucker, message.prompt.clone()).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    info!("{}", format!("✅ Prompt processado com sucesso! Resposta enviada para o bucket S3: {}", mybucker));

    let message_response:MessageResponse = MessageResponse{
        part_text: message.prompt.clone().chars().take(200).collect::<String>(),
        bucket_name: mybucker,
        file_name:file_name_up,
    };

    Ok(Json(message_response))
}