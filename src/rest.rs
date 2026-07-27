//! API REST — servidor HTTP com endpoints para consulta ao modelo Ollama.
//!
//! Expõe um servidor Axum em `http://localhost:8080` com uma rota
//! `/api/prompt` que aceita consultas via GET e retorna respostas
//! geradas pelo modelo com tool calling MCP nativo.
//!
//! # Estrutura
//!
//! - [`router_api`]: configura as rotas do servidor.
//! - [`server_app`]: inicializa o listener HTTP com graceful shutdown.
//! - [`state`]: estado compartilhado com o `McpClientManager`.
//! - [`message`]: estrutura de dados para requisição/resposta.
//! - [`prompt_handler`]: lógica de tratamento do endpoint `/api/prompt`.

pub mod router_api;
pub mod server_app;
pub(crate) mod message;
pub(crate) mod prompt_handler;
pub(crate) mod state;