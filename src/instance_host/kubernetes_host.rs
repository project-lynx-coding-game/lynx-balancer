use crate::instance_host::{Instance, InstanceHost};

use async_trait::async_trait;
use k8s_openapi::api::{batch::v1::Job, core::v1::Pod};
use kube::{
    api::{Api, DeleteParams, ListParams, PostParams},
    runtime::{conditions::is_pod_running, wait::await_condition},
    Client,
};
use tracing::info;

pub struct KubernetesHost {}

impl KubernetesHost {
    pub fn new() -> KubernetesHost {
        KubernetesHost {}
    }
}

#[async_trait]
impl InstanceHost for KubernetesHost {
    async fn start_instance(
        &mut self,
        username: String,
    ) -> Result<Instance, Box<dyn std::error::Error>> {
        let client = Client::try_default().await?;
        let jobs: Api<Job> = Api::default_namespaced(client.clone());

        info!("Creating job for user: {}", username);
        let name = username.clone();
        let data = serde_json::from_value(serde_json::json!({
            "apiVersion": "batch/v1",
            "kind": "Job",
            "metadata": {
                "name": name,
            },
            "spec": {
                "template": {
                    "metadata": {
                        "name": "instance-dynamic-pod"
                    },
                    "spec": {
                        "containers": [{
                            "name": "shell",
                            "image": "busybox",
                            "command": ["/bin/sh",  "-c", "for i in 9 8 7 6 5 4 3 2 1 ; do echo $i ; sleep 100 ; done"]
                        }],
                        "restartPolicy": "Never",
                    }
                }
            }
        }))?;
        jobs.create(&PostParams::default(), &data).await?;

        let pods: Api<Pod> = Api::default_namespaced(client.clone());
        let label = format!("job-name={}", name);
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
        info!("Pod created for {} was created at: {}", username, ip);

        let instance = Instance::new(ip, 8080);

        Ok(instance)
    }

    async fn stop_instance(&self, username: String) -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::try_default().await?;
        let jobs: Api<Job> = Api::default_namespaced(client);

        info!("Cleaning up job for: {}", username);

        let name = username;
        jobs.delete(&name, &DeleteParams::background()).await?;

        Ok(())
    }
}
