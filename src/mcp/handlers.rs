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
            let container_id = arguments["container_id"].as_str().ok_or_else(|| anyhow::anyhow!("ID manquant"))?;
            let tail = arguments["tail"].as_u64().unwrap_or(50) as usize;
            
            let logs = docker.get_container_logs(container_id, tail).await?;
            Ok(json!({ "content": [{ "type": "text", "text": logs }] }))
        }
        "stop_container" => {
            let container_id = arguments["container_id"].as_str().ok_or_else(|| anyhow::anyhow!("ID manquant"))?;
            docker.stop_container(container_id, 10).await?;

            Ok(json!({ "content": [{ "type": "text", "text": format!("Le conteneur {} a été arrêté avec succès.", container_id) }] }))
        }
        _ => Err(anyhow::anyhow!("Outil non trouvé")),
    }
}