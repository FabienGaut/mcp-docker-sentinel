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
        "start_container" => {
            let container_id = arguments["container_id"].as_str().ok_or_else(|| anyhow::anyhow!("ID manquant"))?;
            docker.start_container(container_id).await?;
            Ok(json!({ "content": [{ "type": "text", "text": format!("Le conteneur {} a été démarré avec succès.", container_id) }] }))
        }
        "inspect_container" => {
            let container_id = arguments["container_id"].as_str().ok_or_else(|| anyhow::anyhow!("ID manquant"))?;
            let info = docker.inspect_container(container_id).await?;
            Ok(json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&info)? }] }))
        }
        "restart_container" => {
            let container_id = arguments["container_id"].as_str().ok_or_else(|| anyhow::anyhow!("ID manquant"))?;
            docker.restart_container(container_id, 10).await?;
            Ok(json!({ "content": [{ "type": "text", "text": format!("Le conteneur {} a été redémarré avec succès.", container_id) }] }))
        }
        "remove_container" => {
            let container_id = arguments["container_id"].as_str().ok_or_else(|| anyhow::anyhow!("ID manquant"))?;
            let force = arguments["force"].as_bool().unwrap_or(false);
            docker.remove_container(container_id, force).await?;
            Ok(json!({ "content": [{ "type": "text", "text": format!("Le conteneur {} a été supprimé.", container_id) }] }))
        }
        "list_images" => {
            let images = docker.list_images().await?;
            Ok(json!({ "content": [{ "type": "text", "text": serde_json::to_string(&images)? }] }))
        }
        _ => Err(anyhow::anyhow!("Outil non trouvé")),
    }
}
