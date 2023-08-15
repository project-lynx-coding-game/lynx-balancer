use crate::instance_host::{Instance, InstanceHost};

use async_trait::async_trait;
use k8s_openapi::api::batch::v1::Job;
use kube::{
    api::{Api, DeleteParams, PostParams},
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
        let jobs: Api<Job> = Api::default_namespaced(client);

        info!("Creating job for user: {}", username);
        let name = username;
        let data = serde_json::from_value(serde_json::json!({
            "apiVersion": "batch/v1",
            "kind": "Job",
            "metadata": {
                "name": name,
            },
            "spec": {
                "template": {
                    "metadata": {
                        "name": "empty-job-pod"
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

        // TODO: fetch IP of pod created for the job
        jobs.get(&name).await?;


        let instance = Instance::new(String::from("xd"), 9000);

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
