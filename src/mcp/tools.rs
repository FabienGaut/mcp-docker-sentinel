use serde_json::{json, Value};

pub fn get_tools_definition() -> Value { // format of the tools to call functions
    json!([
        {
            "name": "list_containers",
            "description": "Lists all local Docker containers. Returns their name, image, ID and status (running, exited, etc.).",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        },
        {
            "name": "get_logs",
            "description": "Retrieves the last lines of a container's logs. Useful for diagnosing errors or crashes.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "container_id": {
                        "type": "string",
                        "description": "The name or short ID of the container (e.g. 'my-db')"
                    },
                    "tail": {
                        "type": "integer",
                        "description": "Number of lines to retrieve from the end.",
                        "default": 50
                    }
                },
                "required": ["container_id"]
            }
        },
        {
            "name": "inspect_container",
            "description": "Returns full technical details of a container (Network, Volumes, Environment variables).",
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
            "description": "Stops a container.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "container_id": { "type": "string" }
                },
                "required": ["container_id"]
            }
        },
        {
            "name": "start_container",
            "description": "Starts a container.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "container_id": { "type": "string" }
                },
                "required": ["container_id"]
            }
        },
        {
            "name": "restart_container",
            "description": "Restarts a container (running or stopped).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "container_id": { "type": "string" }
                },
                "required": ["container_id"]
            }
        },
        {
            "name": "remove_container",
            "description": "Removes a container. Use force=true to force removal even if the container is running.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "container_id": { "type": "string" },
                    "force": {
                        "type": "boolean",
                        "description": "Force removal even if the container is running (default: false).",
                        "default": false
                    }
                },
                "required": ["container_id"]
            }
        },
        {
            "name": "list_images",
            "description": "Lists all local Docker images with their ID, tags and size.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        },
        {
            "name": "get_stats",
            "description": "Shows real-time statistics for a container: CPU, memory, network.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "container_id": {
                        "type": "string",
                        "description": "The name or ID of the container"
                    }
                },
                "required": ["container_id"]
            }
        },
        {
            "name": "exec_command",
            "description": "Executes a command inside a running container (equivalent to docker exec).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "container_id": {
                        "type": "string",
                        "description": "The name or ID of the container"
                    },
                    "command": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "The command to execute as an array (e.g. [\"ls\", \"-la\"])"
                    }
                },
                "required": ["container_id", "command"]
            }
        },
        {
            "name": "remove_image",
            "description": "Removes a local Docker image.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "image_id": {
                        "type": "string",
                        "description": "The name or ID of the image to remove"
                    },
                    "force": {
                        "type": "boolean",
                        "description": "Force removal even if the image is in use (default: false).",
                        "default": false
                    }
                },
                "required": ["image_id"]
            }
        },
        {
            "name": "pull_image",
            "description": "Downloads an image from a Docker registry (e.g. docker pull).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "image": {
                        "type": "string",
                        "description": "The name of the image to download (e.g. 'nginx:latest', 'postgres:16')"
                    }
                },
                "required": ["image"]
            }
        },
        {
            "name": "list_networks",
            "description": "Lists Docker networks and the containers connected to them.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        },
        {
            "name": "list_volumes",
            "description": "Lists Docker volumes with their name, driver and mountpoint.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }
    ])
}
