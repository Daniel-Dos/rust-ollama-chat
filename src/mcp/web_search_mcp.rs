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

async fn mcp_client() -> Result<RunningService<RoleClient, InitializeRequestParams>, anyhow::Error> {
    let transport = mcp_transport()?;
    let mcp_client = ClientInfo::default()
        .serve(transport)
        .await
        .context("Failed to serve MCP client")?;
    Ok(mcp_client)
}

#[allow(dead_code)]
async fn tools_list(
    client: &RunningService<RoleClient, InitializeRequestParams>,
) -> Result<Vec<Tool>, anyhow::Error> {
    let tools = client.peer().list_all_tools().await?;

    info!("Discovered {} MCP tool(s)", tools.len());

    Ok(tools)
}

#[allow(dead_code)]
pub struct MCPTools {
    pub tool: Vec<Tool>,
    pub client_mcp: RunningService<RoleClient, InitializeRequestParams>,
}

#[allow(dead_code)]
pub async fn mcp_client_tools() -> Result<MCPTools, anyhow::Error> {
    let mcp_client = mcp_client().await?;
    let tools = tools_list(&mcp_client).await?;
    Ok(MCPTools {
        tool: tools,
        client_mcp: mcp_client,
    })
}

pub struct McpClientManager {
    running_service: RunningService<RoleClient, InitializeRequestParams>,
}

impl McpClientManager {
    pub async fn init() -> Result<Self, anyhow::Error> {
        let running_service = mcp_client().await?;
        Ok(Self { running_service })
    }

    pub fn peer(&self) -> Peer<RoleClient> {
        self.running_service.peer().clone()
    }

    pub async fn shutdown(&mut self) -> Result<(), anyhow::Error> {
        self.running_service.close().await?;
        Ok(())
    }
}
