mod docker;
mod mcp;
mod utils;

use crate::docker::DockerClient;
use crate::mcp::handlers::handle_tool_call;
use crate::mcp::tools::get_tools_definition;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let docker = DockerClient::new().await?;

    let mut reader = BufReader::new(tokio::io::stdin());
    let mut writer = tokio::io::stdout();
    let mut line = String::new();

    loop {
        line.clear();
        if reader.read_line(&mut line).await? == 0 {
            break; // EOF
        }

        let msg: Value = match serde_json::from_str(line.trim()) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Les notifications n'ont pas d'`id` — on ne répond pas
        let id = match msg.get("id") {
            Some(id) => id.clone(),
            None => continue,
        };

        let method = msg["method"].as_str().unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(json!({}));

        let result: Result<Value, String> = match method {
            "initialize" => Ok(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "mcp-docker-sentinel",
                    "version": "0.1.0"
                }
            })),
            "tools/list" => Ok(json!({
                "tools": get_tools_definition()
            })),
            "tools/call" => {
                let name = params["name"].as_str().unwrap_or("");
                let arguments = params["arguments"].clone();
                handle_tool_call(name, arguments, &docker)
                    .await
                    .map_err(|e| e.to_string())
            }
            _ => Err(format!("Method not found: {}", method)),
        };

        let response = match result {
            Ok(result) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            }),
            Err(msg) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": msg
                }
            }),
        };

        let response_str = serde_json::to_string(&response)? + "\n";
        writer.write_all(response_str.as_bytes()).await?;
        writer.flush().await?;
    }

    Ok(())
}
