mod docker;
mod mcp;
mod utils;

use async_trait::async_trait;
use crate::docker::DockerClient;
use crate::mcp::handlers::handle_tool_call;
use crate::mcp::tools::get_tools_definition;
use mcp_sdk_rs::server::{Server, ServerHandler};
use mcp_sdk_rs::error::Error;
use mcp_sdk_rs::types::{ClientCapabilities, Implementation, ServerCapabilities};
use mcp_sdk_rs::transport::stdio::StdioTransport;
use serde_json::json;
use std::sync::Arc;

struct DockerHandler {
    docker: Arc<DockerClient>,
    tools: serde_json::Value,
}

#[async_trait]
impl ServerHandler for DockerHandler {
    async fn initialize(
        &self,
        _implementation: Implementation,
        _capabilities: ClientCapabilities,
    ) -> Result<ServerCapabilities, Error> {
        Ok(ServerCapabilities::default())
    }

    async fn shutdown(&self) -> Result<(), Error> {
        Ok(())
    }

    async fn handle_method(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, Error> {
        match method {
            "tools/list" => Ok(json!({ "tools": self.tools })),
            "tools/call" => {
                let params = params.unwrap_or_default();
                let name = params["name"].as_str().unwrap_or("");
                let arguments = params["arguments"].clone();
                handle_tool_call(name, arguments, &self.docker)
                    .await
                    .map_err(|e| Error::Other(e.to_string()))
            }
            _ => Err(Error::Other(format!("Méthode inconnue: {}", method))),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let docker_client = Arc::new(DockerClient::new().await?);
    let tools = get_tools_definition();

    let handler = Arc::new(DockerHandler { docker: docker_client, tools });
    let (transport, _sender) = StdioTransport::new();
    let server = Server::new(Arc::new(transport), handler);

    server.start().await?;
    Ok(())
}
