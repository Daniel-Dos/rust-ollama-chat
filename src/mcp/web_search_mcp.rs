//! Gerenciador do cliente MCP — conexão com servidor Python de busca web.
//!
//! O módulo gerencia o ciclo de vida de um processo filho Python
//! (`mcp/web-search-mcp.py`) que expõe ferramentas de busca web
//! via protocolo MCP sobre stdio.
//!
//! # Estrutura
//!
//! - [`McpClientManager`]: ponto de entrada principal — inicializa o servidor
//!   Python e expõe um [`Peer<RoleClient>`](rmcp::Peer) para chamadas RPC.
//! - Funções legadas (`MCPTools`, `mcp_client_tools`, `tools_list`) mantidas
//!   para compatibilidade com código legado.

use anyhow::Context;
use rmcp::model::{ClientInfo, InitializeRequestParams, Tool};
use rmcp::service::RunningService;
use rmcp::transport::TokioChildProcess;
use rmcp::{Peer, RoleClient, ServiceExt};

use tokio::process::Command;
use tracing::info;

fn mcp_transport() -> Result<TokioChildProcess, anyhow::Error> {
    let project_dir = std::env::current_dir()?;
    let mcp_script = project_dir.join("mcp/web-search-mcp.py");

    let mut cmd = Command::new("python3");
    cmd.arg(mcp_script.as_path().to_str().unwrap());
    let transport = TokioChildProcess::new(cmd)?;

    Ok(transport)
}

async fn mcp_client() -> Result<RunningService<RoleClient, InitializeRequestParams>, anyhow::Error>
{
    let transport = mcp_transport()?;
    let mcp_client = ClientInfo::default()
        .serve(transport)
        .await
        .context("Failed to serve MCP client")?;
    Ok(mcp_client)
}

/// Lista todas as ferramentas disponíveis no servidor MCP Python.
///
/// **Nota:** Mantida para compatibilidade — não utilizada no fluxo principal atual.
#[allow(dead_code)]
async fn tools_list(
    client: &RunningService<RoleClient, InitializeRequestParams>,
) -> Result<Vec<Tool>, anyhow::Error> {
    let tools = client.peer().list_all_tools().await?;

    info!("Discovered {} MCP tool(s)", tools.len());

    Ok(tools)
}

/// Estrutura legada que agrupa ferramentas MCP descobertas e o cliente.
///
/// **Nota:** Mantida para compatibilidade — não utilizada no fluxo principal atual.
#[allow(dead_code)]
pub struct MCPTools {
    /// Lista de ferramentas disponíveis no servidor.
    pub tool: Vec<Tool>,
    /// Cliente MCP conectado ao servidor Python.
    pub client_mcp: RunningService<RoleClient, InitializeRequestParams>,
}

/// Inicializa o cliente MCP e descobre as ferramentas disponíveis.
///
/// **Nota:** Mantida para compatibilidade — não utilizada no fluxo principal atual.
#[allow(dead_code)]
pub async fn mcp_client_tools() -> Result<MCPTools, anyhow::Error> {
    let mcp_client = mcp_client().await?;
    let tools = tools_list(&mcp_client).await?;
    Ok(MCPTools {
        tool: tools,
        client_mcp: mcp_client,
    })
}

/// Gerenciador da conexão MCP com o servidor Python de busca web.
///
/// # Exemplo
///
/// ```no_run
/// use crate::mcp::web_search_mcp::McpClientManager;
///
/// # async fn example() -> Result<(), anyhow::Error> {
/// let mut manager = McpClientManager::init().await?;
/// let peer = manager.peer();
/// // usa peer para chamar ferramentas MCP...
/// manager.shutdown().await?;
/// # Ok(())
/// # }
/// ```
pub struct McpClientManager {
    running_service: RunningService<RoleClient, InitializeRequestParams>,
}

impl McpClientManager {
    /// Inicializa o servidor Python MCP e estabelece a conexão.
    ///
    /// Dispara o processo `python3 mcp/web-search-mcp.py` e aguarda
    /// o handshake MCP. Deve ser chamada uma única vez no ciclo de vida
    /// da aplicação.
    ///
    /// # Errors
    ///
    /// Retorna erro se o script Python não for encontrado, não puder ser
    /// executado, ou se o handshake MCP falhar.
    pub async fn init() -> Result<Self, anyhow::Error> {
        let running_service = mcp_client().await?;
        Ok(Self { running_service })
    }

    /// Retorna um [`Peer<RoleClient>`](rmcp::Peer) clonado para chamadas RPC.
    ///
    /// O peer pode ser usado para invocar ferramentas MCP como
    /// `web_search` e `web_fetch`.
    pub fn peer(&self) -> Peer<RoleClient> {
        self.running_service.peer().clone()
    }

    /// Retorna todas as ferramentas disponíveis no servidor MCP.
    ///
    /// # Errors
    ///
    /// Retorna erro se a chamada RPC falhar.
    pub async fn get_tools(&self) -> Result<Vec<Tool>, anyhow::Error> {
        let tools = self.running_service.peer().list_all_tools().await?;
        info!("Discovered {} MCP tool(s)", tools.len());
        Ok(tools)
    }

    /// Retorna o `Peer<RoleClient>` para uso como sink com `rig-core` rmcp tools.
    ///
    /// O peer implementa o trait `Sink` necessário para o rig-core se comunicar
    /// com o servidor MCP ao chamar tools via `AgentBuilder::rmcp_tools()`.
    pub fn get_sink(&self) -> Peer<RoleClient> {
        self.running_service.peer().clone()
    }

    /// Encerra a conexão com o servidor Python MCP.
    ///
    /// Fecha o processo filho de forma limpa. Deve ser chamada antes
    /// do encerramento da aplicação.
    ///
    /// # Errors
    ///
    /// Retorna erro se o fechamento do processo falhar.
    pub async fn shutdown(&mut self) -> Result<(), anyhow::Error> {
        self.running_service.close().await?;
        Ok(())
    }
}
