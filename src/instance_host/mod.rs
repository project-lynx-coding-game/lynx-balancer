pub mod kubernetes_host;
pub mod local_host;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Instance {
    pub url: String,
    pub port: u16,
}

impl Instance {
    pub fn new(url: String, port: u16) -> Instance {
        Instance { url, port }
    }

    pub fn get_url_with_port(&self) -> String {
        self.url.clone() + ":" + &self.port.clone().to_string()
    }
}

#[async_trait]
pub trait InstanceHost {
    async fn start_instance(
        &mut self,
        username: String,
    ) -> Result<Instance, Box<dyn std::error::Error>>;
    async fn stop_instance(&mut self, username: String) -> Result<(), Box<dyn std::error::Error>>;
}
