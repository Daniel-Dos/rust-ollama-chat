//! Cliente MCP (Model Context Protocol) para comunicação com servidor Python.
//!
//! Gerencia o ciclo de vida do processo filho Python que expõe ferramentas
//! de busca web (`web_search`, `web_fetch`) via protocolo MCP sobre stdio.

pub mod web_search_mcp;
