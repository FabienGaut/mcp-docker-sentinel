use crate::docker::DockerClient;
use serde_json::{json, Value};
use anyhow::Result;

pub async fn handle_tool_call(
    tool_name: &str,
    arguments: Value,
    docker: &DockerClient,
) -> Result<Value> {
    match tool_name {
        "list_containers" => {
            let containers = docker.list_all_containers().await?;
            Ok(json!({ "content": [{ "type": "text", "text": serde_json::to_string(&containers)? }] }))
        }
        "get_logs" => {
            let container_id = arguments["container_id"].as_str().ok_or_else(|| anyhow::anyhow!("Missing ID"))?;
            let tail = arguments["tail"].as_u64().unwrap_or(50) as usize;
            
            let logs = docker.get_container_logs(container_id, tail).await?;
            Ok(json!({ "content": [{ "type": "text", "text": logs }] }))
        }
        "stop_container" => {
            let container_id = arguments["container_id"].as_str().ok_or_else(|| anyhow::anyhow!("Missing ID"))?;
            docker.stop_container(container_id, 10).await?;

            Ok(json!({ "content": [{ "type": "text", "text": format!("Container {} has been stopped successfully.", container_id) }] }))
        }
        "start_container" => {
            let container_id = arguments["container_id"].as_str().ok_or_else(|| anyhow::anyhow!("Missing ID"))?;
            docker.start_container(container_id).await?;
            Ok(json!({ "content": [{ "type": "text", "text": format!("Container {} has been started successfully.", container_id) }] }))
        }
        "inspect_container" => {
            let container_id = arguments["container_id"].as_str().ok_or_else(|| anyhow::anyhow!("Missing ID"))?;
            let info = docker.inspect_container(container_id).await?;
            Ok(json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&info)? }] }))
        }
        "restart_container" => {
            let container_id = arguments["container_id"].as_str().ok_or_else(|| anyhow::anyhow!("Missing ID"))?;
            docker.restart_container(container_id, 10).await?;
            Ok(json!({ "content": [{ "type": "text", "text": format!("Container {} has been restarted successfully.", container_id) }] }))
        }
        "remove_container" => {
            let container_id = arguments["container_id"].as_str().ok_or_else(|| anyhow::anyhow!("Missing ID"))?;
            let force = arguments["force"].as_bool().unwrap_or(false);
            docker.remove_container(container_id, force).await?;
            Ok(json!({ "content": [{ "type": "text", "text": format!("Container {} has been removed.", container_id) }] }))
        }
        "list_images" => {
            let images = docker.list_images().await?;
            Ok(json!({ "content": [{ "type": "text", "text": serde_json::to_string(&images)? }] }))
        }
        "get_stats" => {
            let container_id = arguments["container_id"].as_str().ok_or_else(|| anyhow::anyhow!("Missing ID"))?;
            let stats = docker.get_stats(container_id).await?;
            Ok(json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&stats)? }] }))
        }
        "exec_command" => {
            let container_id = arguments["container_id"].as_str().ok_or_else(|| anyhow::anyhow!("Missing ID"))?;
            let cmd: Vec<String> = arguments["command"]
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("Missing command (expected array of strings)"))?
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
            let output = docker.exec_command(container_id, cmd).await?;
            Ok(json!({ "content": [{ "type": "text", "text": output }] }))
        }
        "remove_image" => {
            let image_id = arguments["image_id"].as_str().ok_or_else(|| anyhow::anyhow!("Missing image ID"))?;
            let force = arguments["force"].as_bool().unwrap_or(false);
            docker.remove_image(image_id, force).await?;
            Ok(json!({ "content": [{ "type": "text", "text": format!("Image {} has been removed.", image_id) }] }))
        }
        "pull_image" => {
            let image = arguments["image"].as_str().ok_or_else(|| anyhow::anyhow!("Missing image name"))?;
            let output = docker.pull_image(image).await?;
            Ok(json!({ "content": [{ "type": "text", "text": output }] }))
        }
        "list_networks" => {
            let networks = docker.list_networks().await?;
            Ok(json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&networks)? }] }))
        }
        "list_volumes" => {
            let volumes = docker.list_volumes().await?;
            Ok(json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&volumes)? }] }))
        }
        _ => Err(anyhow::anyhow!("Tool not found")),
    }
}
