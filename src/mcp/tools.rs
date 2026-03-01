use serde_json::{json, Value};

pub fn get_tools_definition() -> Value {
    json!([
        {
            "name": "list_containers",
            "description": "Liste tous les conteneurs Docker locaux. Retourne leur nom, image, ID et statut (running, exited, etc.).",
            "inputSchema": {
                "type": "object",
                "properties": {}, // Pas d'arguments nécessaires
                "required": []
            }
        },
        {
            "name": "get_logs",
            "description": "Récupère les dernières lignes de log d'un conteneur. Peut permettre de diagnostiquer une erreur ou un crash.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "container_id": {
                        "type": "string",
                        "description": "Le nom ou l'ID court du conteneur (ex: 'my-db')"
                    },
                    "tail": {
                        "type": "integer",
                        "description": "Nombre de lignes à récupérer en partant de la fin.",
                        "default": 50
                    }
                },
                "required": ["container_id"]
            }
        },
        {
            "name": "inspect_container",
            "description": "Donne les détails techniques complets d'un conteneur (Réseau, Volumes, Variables d'environnement).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "container_id": { "type": "string" }
                },
                "required": ["container_id"]
            }
        },
        {
            "name": "stop_container",
            "description": "Arrête un conteneur.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "container_id": { "type": "string" }
                },
                "required": ["container_id"]
            }
        }
    ])
}