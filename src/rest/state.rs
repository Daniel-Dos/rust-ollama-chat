//! Estado compartilhado da aplicação, injetado via Axum extractor.

use crate::mcp::web_search_mcp::McpClientManager;
use crate::rest::message::Message;
use std::sync::Arc;

/// Estado global da aplicação REST, compartilhado entre handlers.
///
/// Contém o gerenciador MCP responsável pela comunicação com o
/// servidor Python de busca web, encapsulado em `Arc` para
/// acesso concorrente.
#[derive(Clone)]
pub struct AppState {
    /// Gerenciador da conexão MCP com o servidor Python de busca web.
    pub mcp_manager: Arc<McpClientManager>,
}