use crate::instance_host::{Instance, InstanceHost};

use kube::{Client, Config, client::ConfigExt, api::{Api, ResourceExt, ListParams, PostParams}};
use k8s_openapi::api::core::v1::Pod;
use async_trait::async_trait;
use tower::ServiceBuilder;
use k8s_openapi::http::Uri;

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
        let config = Config::new("server.kubernetes.blazej-smorawski.com".parse::<Uri>().unwrap());
        let service = ServiceBuilder::new()
            .layer(config.base_uri_layer())
            .option_layer(config.auth_layer()?)
            .service(hyper::Client::new());
        let client = Client::new(service, config.default_namespace);

        let pods: Api<Pod> = Api::default_namespaced(client);
        for p in pods.list(&ListParams::default()).await? {
            println!("found pod {}", p.name_any());
        }

        let instance = Instance::new(String::from("xd"), 9000);
        self.instances.push(Box::new(instance.clone()));

        println!("{:?}", instance);
        Ok(instance)
    }

    fn stop_instance(&self, instance: Instance) {
        println!("{:?}", instance);
        todo!()
    }
}
