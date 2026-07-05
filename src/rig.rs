//! Núcleo da aplicação — integração com Ollama e busca web via MCP.
//!
//! Este módulo contém a lógica de chat com modelos de linguagem (Ollama)
//! e a comunicação com o servidor MCP Python para busca e fetch de URLs.

pub mod client_ollama;
pub mod web_search;
