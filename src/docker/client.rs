use bollard::Docker;
use bollard::container::{ListContainersOptions, LogsOptions, StopContainerOptions};
use futures_util::StreamExt;
use anyhow::Result; 

pub struct DockerClient {
    // permet de parler au démon Docker
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
            tail: limit.to_string(), // tail recupere la fin d'un log pour en avoir le contenu résumé
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
            status: c.status.unwrap_or_else(|| "Inconnu".to_string()),
            image: c.image.unwrap_or_default(),
        }).collect())
    }

    pub async fn stop_container(&self, name: &str, timeout_seconds: i64) -> Result<()> {
        let options = StopContainerOptions {
            t: timeout_seconds, // temps avant de stopper le container
        };
        self.inner.stop_container(name, Some(options)).await?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
pub struct Container {
    pub name: String,
    pub status: String,
    pub image: String,
}