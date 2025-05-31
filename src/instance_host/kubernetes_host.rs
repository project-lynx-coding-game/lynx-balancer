use crate::instance_host::{Instance, InstanceHost};

use async_trait::async_trait;
use k8s_openapi::api::{batch::v1::Job, core::v1::Pod};
use kube::{
    api::{Api, DeleteParams, ListParams, PostParams},
    runtime::{conditions::is_pod_running, wait::await_condition},
};
use tracing::info;

pub struct KubernetesHost {}

impl KubernetesHost {
    pub fn new() -> KubernetesHost {
        KubernetesHost {}
    }
    // TODO: we assume that job name = username. It probably should not be the case later. But it's ok for now.
    async fn create_job(
        &self,
        username: String,
        client: kube::Client,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Creating job for user: {}", username);
        let jobs: Api<Job> = Api::default_namespaced(client);
        let data = serde_json::from_value(serde_json::json!({
            "apiVersion": "batch/v1",
            "kind": "Job",
            "metadata": {
                "name": username,
            },
            "spec": {
                "template": {
                    "metadata": {
                        "name": "instance-dynamic-pod"
                    },
                    "spec": {
                        "containers": [{
                            "name": "scene-host",
                            "image": "ghcr.io/group-project-gut/lynx-scene-host-python:latest",
                            "args": ["main:app", "--port", "8080", "--host", "0.0.0.0", "--workers", "1"],
                            "ports": [{"containerPort": 8080}],
                            "env": [{
                                "name": "LYNX_SCENE_GENERATOR_URL",
                                "value":"http://lynx-scene-generator-service.lynx-scene-generator:8080/get_scene"
                            }]
                        }],
                        "restartPolicy": "Never",
                    }
                }
            }
        }))?;
        jobs.create(&PostParams::default(), &data).await?;
        Ok(())
    }

    async fn get_job_ip(
        &self,
        username: String,
        client: kube::Client,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let pods: Api<Pod> = Api::default_namespaced(client);
        let label = format!("job-name={}", username);
        let lp = ListParams::default().labels(&label);
        let mut name = "".to_string();

        while name == "" {
            for p in pods.list(&lp).await? {
                if p.metadata.deletion_timestamp.is_some() {
                    // Pod is terminating
                    continue;
                }

                name = p.metadata.name.unwrap();
                info!("Pod name is: {}", name);
                break;
            }
        }

        let running = await_condition(pods.clone(), &name, is_pod_running());
        let _ = tokio::time::timeout(std::time::Duration::from_secs(15), running).await?;

        let p = pods.get(&name).await?;
        let status = p.status.unwrap();
        let ip = status.pod_ip.unwrap();
        info!(
            "Pod created for {} was created at: {}:{}",
            username, ip, 8080
        );
        Ok(ip)
    }
}

#[async_trait]
impl InstanceHost for KubernetesHost {
    async fn start_instance(
        &mut self,
        username: String,
    ) -> Result<Instance, Box<dyn std::error::Error>> {
        let client = kube::Client::try_default().await?;

        self.create_job(username.clone(), client.clone()).await?;
        let ip: String = self.get_job_ip(username.clone(), client.clone()).await?;

        let instance = Instance::new(ip, 8080);

        Ok(instance)
    }

    async fn stop_instance(&mut self, username: String) -> Result<(), Box<dyn std::error::Error>> {
        let client = kube::Client::try_default().await?;
        let jobs: Api<Job> = Api::default_namespaced(client);

        info!("Cleaning up job for: {}", username);

        let name = username;
        jobs.delete(&name, &DeleteParams::background()).await?;

        Ok(())
    }
}
