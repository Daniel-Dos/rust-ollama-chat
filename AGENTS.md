# AGENTS.md — Rust-Rig-AI

Repositório monobinário Rust (edition 2024) que integra Ollama + busca web via MCP + upload S3.

## Comandos essenciais

- **Build:** `cargo build`
- **Lint (modo estrito):** `cargo clippy --all-targets --all-features --locked -- -D warnings`
- **Testar:** `cargo test`
- Sem `rustfmt.toml`, `justfile`, `Makefile` ou scripts npm — use comandos Cargo padrão.

## Entrypoint e fluxo

```
src/main.rs
  ├── McpClientManager::init()     → sobe Python MCP server uma vez
  ├── banner_text::print_banner()
  ├── rust_ollama::resposta_chat(&mcp_manager, prompt)
  │     ├── extrair_url(prompt)    → regex decide se é fetch direto ou busca
  │     ├── fetch_url(peer, url)   → se tem URL
  │     └── search_and_fetch(peer, q) → busca + fetch do primeiro resultado
  ├── upload resposta para S3
  └── mcp_manager.shutdown()
```

## Variáveis de ambiente obrigatórias

| Variável | Obrigatória? | Uso |
|----------|-------------|------|
| `OLLAMA_MODEL` | **Sim** | Modelo Ollama (ex: `gemma2:9b`, `llama3.2`) |
| `OLLAMA_API_KEY` | **Sim** | API key do Ollama (usada pelo Python MCP server p/ busca web) |
| `RUST_LOG` | Não | Filtro tracing (default: `warn,Rust_Rig_AI=info`) |

## Arquitetura e convenções

- **Conexão MCP é reusada:** criar `McpClientManager::init()` uma vez em `main.rs`, passar `&McpClientManager` como dependência.
- **Detecção URL vs busca:** usar `extrair_url(prompt)`. Se retornar URLs → `fetch_url`; se vazio → `search_and_fetch`. **Não** usar `contains("http")`.
- **AWS S3:** usa o primeiro bucket da conta, gera nome aleatório. Requer `floci` (Docker):
  ```bash
  docker compose up -d floci
  ```
- **Módulos:** `rig/` = core (chat + web search), `mcp/` = cliente MCP, `aws/` = S3, `banner/` = ASCII art.
- **`Cargo.lock` versionado** (não está no `.gitignore`). Builds são reproduzíveis.

## MCP server (Python)

`mcp/web-search-mcp.py` é spawnado automaticamente. Dependências:
```bash
pip install -r mcp/requirements.txt   # mcp>=1.0.0, ollama>=0.4.0, rich>=13.0.0
```

Sem hot-reload — reinicie a aplicação se editar o Python.

## Código legado com `#[allow(dead_code)]`

`MCPTools`, `tools_list()`, `mcp_client_tools()`, `search_web()` — backward compat. Não remover sem verificar dependências externas.

## Erros comuns

- **`OLLAMA_MODEL` não definida** → `resposta_chat` retorna erro.
- **`OLLAMA_API_KEY` não definida** → busca web falha silenciosamente no Python MCP.
- **MCP não sobe** → verificar `python3` + `pip install -r mcp/requirements.txt`.
- **S3 falha** → verificar se `floci` está rodando ou configurar credenciais AWS.
