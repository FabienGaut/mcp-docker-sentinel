mod docker;
mod mcp;
mod utils;

use crate::docker::DockerClient;
use crate::mcp::handlers::handle_tool_call;
use crate::mcp::tools::get_tools_definition;
use serde_json::{json, Value}; // serde_json is used to handle JSON serialization and deserialization
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let docker = DockerClient::new().await?;

    let mut reader = BufReader::new(tokio::io::stdin()); // async reader for stdin
    let mut writer = tokio::io::stdout(); // async writer for stdout
    let mut line = String::new();

    loop { // infinite loop to read JSON-RPC requests until break
        // Json RPC is a communication protocol used here to call functions with a JSON format command
        // MCP uses this protocol to call the tools from an AI system such as Claude code for example
        line.clear();
        if reader.read_line(&mut line).await? == 0 {
            break; // EOF
        }

        let msg: Value = match serde_json::from_str(line.trim()) { // get the json message from the line read
            Ok(v) => v,
            Err(_) => continue,
        };

        let id = match msg.get("id") { // get the id from the json message
            Some(id) => id.clone(),
            None => continue,
        };

        let method = msg["method"].as_str().unwrap_or(""); // get the method from the json message
        let params = msg.get("params").cloned().unwrap_or(json!({})); // get the params from the json message

        let result: Result<Value, String> = match method { // match the method to call the corresponding function
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
            "tools/call" => { // call a defined function with the name and arguments given in the params of the json message
                let name = params["name"].as_str().unwrap_or("");
                let arguments = params["arguments"].clone();
                handle_tool_call(name, arguments, &docker)
                    .await
                    .map_err(|e| e.to_string())
            }
            _ => Err(format!("Method not found: {}", method)),
        };

        let response = match result { // match the result to create the corresponding JSON-RPC response
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

        let response_str = serde_json::to_string(&response)? + "\n"; // convert the response to a JSON string
        writer.write_all(response_str.as_bytes()).await?;
        writer.flush().await?;
    }

    Ok(())
}
