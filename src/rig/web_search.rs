//! Integração com ferramentas de busca web do servidor MCP Python.
//!
//! Fornece funções para extrair URLs de textos, buscar na web e obter
//! o conteúdo completo de páginas via chamadas MCP (`web_search`, `web_fetch`).

use regex::Regex;
use rmcp::model::CallToolRequestParams;
use rmcp::{Peer, RoleClient};
use serde_json::Value;
use tracing::info;

fn extrair_texto(result: &rmcp::model::CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| {
            let raw = c.as_text()?;
            Some(raw.text.clone())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Extrai URLs de uma string usando regex.
///
/// Retorna uma lista com todas as URLs HTTP/HTTPS encontradas no texto.
///
/// # Exemplo
///
/// ```
/// let urls = extrair_url("Veja https://example.com e http://teste.org");
/// assert_eq!(urls.len(), 2);
/// ```
pub fn extrair_url(url: &str) -> Vec<String> {
    let re = Regex::new(r#"https?://[a-zA-Z0-9./?=_%:&#-]+"#)
        .unwrap_or_else(|_| panic!("Falha ao compilar a expressão regular para extrair URLs"));
    re.find_iter(url).map(|m| m.as_str().into()).collect()
}

/// Executa uma busca web via ferramenta MCP `web_search`.
///
/// Envia a consulta ao servidor Python que realiza a busca e retorna
/// os resultados como texto.
///
/// **Nota:** Esta função está marcada como `#[allow(dead_code)]` —
/// mantida para compatibilidade, mas não utilizada no fluxo principal.
#[allow(dead_code)]
pub async fn search_web(peer: &Peer<RoleClient>, query: &str) -> Result<String, anyhow::Error> {
    let args = serde_json::json!({ "query": query, "max_results": 5 })
        .as_object()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("expected JSON object from inline json macro"))?;

    let params = CallToolRequestParams::new("web_search").with_arguments(args);
    let result = peer.call_tool(params).await?;

    let texto = extrair_texto(&result);

    info!("Web search concluído");

    Ok(texto)
}

/// Obtém o conteúdo de uma URL via ferramenta MCP `web_fetch`.
///
/// O servidor Python baixa a página e retorna seu conteúdo textual.
pub async fn fetch_url(peer: &Peer<RoleClient>, url: &str) -> Result<String, anyhow::Error> {
    let args = serde_json::json!({ "url": url })
        .as_object()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("expected JSON object from inline json macro"))?;

    let params = CallToolRequestParams::new("web_fetch").with_arguments(args);
    let result = peer.call_tool(params).await?;

    let texto = extrair_texto(&result);

    info!("Web fetch concluído");

    Ok(texto)
}

/// Combina busca web com fetch da primeira página encontrada.
///
/// 1. Executa `web_search` para encontrar resultados.
/// 2. Extrai a URL do primeiro resultado do JSON retornado.
/// 3. Executa `web_fetch` para obter o conteúdo completo.
/// 4. Retorna ambos os resultados formatados em um único texto.
pub async fn search_and_fetch(
    peer: &Peer<RoleClient>,
    query: &str,
) -> Result<String, anyhow::Error> {
    // 1. Search
    let search_args = serde_json::json!({ "query": query, "max_results": 5 })
        .as_object()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("expected JSON object from inline json macro"))?;

    let search_params = CallToolRequestParams::new("web_search").with_arguments(search_args);
    let search_result = peer.call_tool(search_params).await?;
    let search_texto = extrair_texto(&search_result);

    info!("Search concluído");

    // 2. Parse JSON to extract first URL
    let first_url = serde_json::from_str::<Value>(&search_texto)
        .ok()
        .and_then(|v| {
            v["results"]
                .as_array()?
                .first()?
                .get("url")?
                .as_str()
                .map(String::from)
        });

    // 3. Fetch if URL found
    let fetch_texto = if let Some(url) = &first_url {
        info!("Fetching: {url}");
        let fetch_args = serde_json::json!({ "url": url })
            .as_object()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("expected JSON object from inline json macro"))?;
        let fetch_params = CallToolRequestParams::new("web_fetch").with_arguments(fetch_args);
        let fetch_result = peer.call_tool(fetch_params).await?;
        extrair_texto(&fetch_result)
    } else {
        String::new()
    };

    // 4. Format combined result
    let mut output = format!("Resultados da busca:\n{search_texto}");
    if !fetch_texto.is_empty() {
        output.push_str(&format!(
            "\n\n---\n\nConteúdo completo da página principal:\n{fetch_texto}"
        ));
    }
    Ok(output)
}
