use bollard::Docker;
use bollard::container::{InspectContainerOptions, ListContainersOptions, LogsOptions, RemoveContainerOptions, RestartContainerOptions, StartContainerOptions, StopContainerOptions, StatsOptions};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::{ListImagesOptions, RemoveImageOptions};
use bollard::network::ListNetworksOptions;
use bollard::volume::ListVolumesOptions;
use serde_json::Value;
use futures_util::StreamExt;
use anyhow::Result;

pub struct DockerClient {
    // communicates with the Docker daemon
    inner: Docker,
}

impl DockerClient {
    pub async fn new() -> Result<Self> {
        let client = Docker::connect_with_local_defaults()?;
        Ok(Self { inner: client })
    }

    pub async fn get_container_logs(&self, name: &str, limit: usize) -> Result<String> {
        let options = LogsOptions {
            stdout: true,
            stderr: true,
            tail: limit.to_string(), // tail retrieves the last N lines of a log
            ..Default::default()
        };

        let mut logs_stream = self.inner.logs(name, Some(options));
        let mut output = String::new();

        while let Some(log_result) = logs_stream.next().await {
            match log_result? {
                bollard::container::LogOutput::StdOut { message } |
                bollard::container::LogOutput::StdErr { message } => {
                    output.push_str(&String::from_utf8_lossy(&message));
                }
                _ => {}
            }
        }
        Ok(output)
    }


    pub async fn list_all_containers(&self) -> Result<Vec<Container>> {
        let options = Some(ListContainersOptions::<String> {
            all: true, 
            ..Default::default()
        });

        let containers = self.inner.list_containers(options).await?;

        Ok(containers.into_iter().map(|c| Container { 
            name: c.names.unwrap_or_default().join(", "),
            status: c.status.unwrap_or_else(|| "Unknown".to_string()),
            image: c.image.unwrap_or_default(),
        }).collect())
    }

    pub async fn stop_container(&self, name: &str, timeout_seconds: i64) -> Result<()> {
        let options = StopContainerOptions {
            t: timeout_seconds, // timeout before stopping the container
        };
        self.inner.stop_container(name, Some(options)).await?;
        Ok(())
    }

    pub async fn start_container(&self, name: &str) -> Result<()> {
        let options: Option<StartContainerOptions<String>> = None;
        self.inner.start_container(name, options).await?;
        Ok(())
    }

    pub async fn inspect_container(&self, name: &str) -> Result<Value> {
        let options: Option<InspectContainerOptions> = None;
        let info = self.inner.inspect_container(name, options).await?;
        Ok(serde_json::to_value(info)?)
    }

    pub async fn restart_container(&self, name: &str, timeout_seconds: i64) -> Result<()> {
        let options = RestartContainerOptions { t: timeout_seconds as isize };
        self.inner.restart_container(name, Some(options)).await?;
        Ok(())
    }

    pub async fn remove_container(&self, name: &str, force: bool) -> Result<()> {
        let options = RemoveContainerOptions {
            force,
            ..Default::default()
        };
        self.inner.remove_container(name, Some(options)).await?;
        Ok(())
    }

    pub async fn get_stats(&self, name: &str) -> Result<Value> {
        let options = StatsOptions {
            stream: false,
            one_shot: true,
        };
        let mut stream = self.inner.stats(name, Some(options));
        if let Some(stats) = stream.next().await {
            let stats = stats?;
            let cpu_delta = stats.cpu_stats.cpu_usage.total_usage as f64
                - stats.precpu_stats.cpu_usage.total_usage as f64;
            let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0) as f64
                - stats.precpu_stats.system_cpu_usage.unwrap_or(0) as f64;
            let num_cpus = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;
            let cpu_percent = if system_delta > 0.0 {
                (cpu_delta / system_delta) * num_cpus * 100.0
            } else {
                0.0
            };

            let mem_usage = stats.memory_stats.usage.unwrap_or(0);
            let mem_limit = stats.memory_stats.limit.unwrap_or(1);
            let mem_percent = (mem_usage as f64 / mem_limit as f64) * 100.0;

            let mut rx_bytes: u64 = 0;
            let mut tx_bytes: u64 = 0;
            if let Some(networks) = &stats.networks {
                for net in networks.values() {
                    rx_bytes += net.rx_bytes;
                    tx_bytes += net.tx_bytes;
                }
            }

            Ok(serde_json::json!({
                "cpu_percent": format!("{:.2}", cpu_percent),
                "memory_usage_mb": format!("{:.1}", mem_usage as f64 / 1_048_576.0),
                "memory_limit_mb": format!("{:.1}", mem_limit as f64 / 1_048_576.0),
                "memory_percent": format!("{:.2}", mem_percent),
                "network_rx_mb": format!("{:.2}", rx_bytes as f64 / 1_048_576.0),
                "network_tx_mb": format!("{:.2}", tx_bytes as f64 / 1_048_576.0),
            }))
        } else {
            Err(anyhow::anyhow!("Failed to retrieve container stats"))
        }
    }

    pub async fn exec_command(&self, name: &str, cmd: Vec<String>) -> Result<String> {
        let exec = self.inner.create_exec(name, CreateExecOptions {
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            cmd: Some(cmd),
            ..Default::default()
        }).await?;

        let mut output = String::new();
        if let StartExecResults::Attached { output: mut exec_output, .. } = self.inner.start_exec(&exec.id, None).await? {
            while let Some(chunk) = exec_output.next().await {
                let msg: bollard::container::LogOutput = chunk?;
                output.push_str(&msg.to_string());
            }
        }
        Ok(output)
    }

    pub async fn remove_image(&self, name: &str, force: bool) -> Result<()> {
        let options = RemoveImageOptions {
            force,
            ..Default::default()
        };
        self.inner.remove_image(name, Some(options), None).await?;
        Ok(())
    }

    pub async fn pull_image(&self, image: &str) -> Result<String> {
        use bollard::image::CreateImageOptions;
        let options = CreateImageOptions {
            from_image: image,
            ..Default::default()
        };
        let mut stream = self.inner.create_image(Some(options), None, None);
        let mut output = String::new();
        while let Some(result) = stream.next().await {
            let info = result?;
            if let Some(status) = info.status {
                output.push_str(&status);
                output.push('\n');
            }
        }
        Ok(output)
    }

    pub async fn list_networks(&self) -> Result<Value> {
        let options: ListNetworksOptions<String> = Default::default();
        let networks = self.inner.list_networks(Some(options)).await?;
        let result: Vec<Value> = networks.into_iter().map(|n| {
            let containers: Vec<String> = n.containers
                .map(|c| c.keys().cloned().collect())
                .unwrap_or_default();
            serde_json::json!({
                "name": n.name.unwrap_or_default(),
                "id": n.id.unwrap_or_default(),
                "driver": n.driver.unwrap_or_default(),
                "scope": n.scope.unwrap_or_default(),
                "containers": containers,
            })
        }).collect();
        Ok(serde_json::json!(result))
    }

    pub async fn list_volumes(&self) -> Result<Value> {
        let options: ListVolumesOptions<String> = Default::default();
        let response = self.inner.list_volumes(Some(options)).await?;
        let volumes: Vec<Value> = response.volumes.unwrap_or_default().into_iter().map(|v| {
            serde_json::json!({
                "name": v.name,
                "driver": v.driver,
                "mountpoint": v.mountpoint,
            })
        }).collect();
        Ok(serde_json::json!(volumes))
    }

    pub async fn list_images(&self) -> Result<Vec<Image>> {
        let options = Some(ListImagesOptions::<String> {
            all: false,
            ..Default::default()
        });
        let images = self.inner.list_images(options).await?;
        Ok(images.into_iter().map(|i| Image {
            id: i.id,
            tags: i.repo_tags,
            size: i.size,
        }).collect())
    }
}

#[derive(serde::Serialize)]
pub struct Container {
    pub name: String,
    pub status: String,
    pub image: String,
}

#[derive(serde::Serialize)]
pub struct Image {
    pub id: String,
    pub tags: Vec<String>,
    pub size: i64,
}