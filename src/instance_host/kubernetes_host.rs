use crate::instance_host::{Instance, InstanceHost};

use kube::{Client, api::{Api, ResourceExt, ListParams, PostParams}};
use k8s_openapi::api::core::v1::Pod;
use async_trait::async_trait;

pub struct KubernetesHost {
    instances: Vec<Box<Instance>>
}

impl KubernetesHost {
    pub fn new() -> KubernetesHost {
        KubernetesHost {
            instances: Vec::new()
        }
    }
}

#[async_trait]
impl InstanceHost for KubernetesHost {
    async fn start_instance(&mut self) -> Result<Instance, Box<dyn std::error::Error>> {
        let client = Client::try_default().await?;

        let pods: Api<Pod> = Api::default_namespaced(client);
        for p in pods.list(&ListParams::default()).await? {
            println!("found pod {}", p.name_any());
        }

        let instance = Instance::new(String::from("xd"), 9000);
        self.instances.push(Box::new(instance.clone()));
        Ok(instance)
    }

    fn stop_instance(&self, instance: Instance) {
        println!("{:?}", instance);
        todo!()
    }
}
