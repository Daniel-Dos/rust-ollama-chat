# AGENTS.md — Rust-Rig-AI

Repositório monobinário Rust (edition 2024) que integra Ollama + busca web via MCP + upload S3.

## Comandos essenciais

- **Build:** `cargo build`
- **Lint (modo estrito):** `cargo clippy --all-targets --all-features --locked -- -D warnings`
- **Testar:** `cargo test`
- Sem `rustfmt.toml`, `justfile`, `Makefile` ou scripts npm — use comandos Cargo padrão.

## Entrypoint e fluxo

```
src/main.rs (fn main — sem #[tokio::main])
  ├── tokio::runtime::Runtime::new() + rt.enter()
  ├── McpClientManager::init()     → sobe Python MCP server uma vez (via rt.block_on)
  ├── banner_text::print_banner()
  ├── println!("Iniciando interface gráfica...")
  ├── std::thread::sleep(3s)       ← delay terminal antes da janela
  ├── gui::app::run(&mcp_manager)  ← janela Iced
  │     ├── Splash 5s com barra de progresso animada (█░)
  │     ├── toggle "Buscar na web" + text_input + "Enviar"
  │     ├── mostra "Processando..." (amarelo) durante inferência
  │     ├── Task::perform bifurcada:
  │     │     ├── search=true  → resposta_chat_peer(peer, prompt)
  │     │     └── search=false → chat_direct(prompt)
  │     ├── resposta renderizada com markdown + botão "Copiar"
  │     ├── rodapé: "Tokens: X total — entrada: Y (Z%)  saída: W (V%)"
  │     └── devolve resposta para main quando fecha
  ├── mcp_manager.shutdown()
  └── s3::get_my_bucket() + upload_bucket()
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
- **Módulos:** `rig/` = core (chat + web search), `mcp/` = cliente MCP, `aws/` = S3, `banner/` = ASCII art, `gui/` = GUI Iced.
- **`Cargo.lock` versionado** (não está no `.gitignore`). Builds são reproduzíveis.

## MCP server (Python)

`mcp/web-search-mcp.py` é spawnado automaticamente. Dependências:
```bash
pip install -r mcp/requirements.txt   # mcp>=1.0.0, ollama>=0.4.0, rich>=13.0.0
```

Sem hot-reload — reinicie a aplicação se editar o Python.

## Token tracking com `extended_details()`

Para obter contagem de tokens na resposta, usa-se o método encadeado:

```rust
use rig_core::agent::PromptResponse;

let agente: AgentMCP = rust_agente().await?;
let pendente = agente.prompt(prompt_final);
let resp: PromptResponse = pendente.extended_details().await?;
// resp.usage.input_tokens, resp.usage.output_tokens, resp.output
```

- `agent.prompt(x)` retorna `PromptRequest<Standard, M, P>`
- `.extended_details()` converte para `PromptRequest<Extended, M, P>`
- `.await?` retorna `PromptResponse { output, usage, ... }` via `IntoFuture`
- A quebra em variáveis é necessária para o rust-analyzer conseguir navegar (genéricos encadeados não são resolvidos pelo LSP)
- **Não** usar `.prompt(x).await` (retorna `String` sem metadados)

## Código legado com `#[allow(dead_code)]`

`MCPTools`, `tools_list()`, `mcp_client_tools()`, `search_web()` — backward compat. Não remover sem verificar dependências externas.

## Erros comuns

- **`OLLAMA_MODEL` não definida** → `resposta_chat_peer` / `chat_direct` retornam erro.
- **`OLLAMA_API_KEY` não definida** → busca web falha silenciosamente no Python MCP.
- **MCP não sobe** → verificar `python3` + `pip install -r mcp/requirements.txt`.
- **S3 falha** → verificar se `floci` está rodando ou configurar credenciais AWS.
