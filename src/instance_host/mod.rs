pub mod kubernetes_host;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Instance {
    url: String,
    port: u16,
}

impl Instance {
    pub fn new(url: String, port: u16) -> Instance {
        Instance {
            url, port
        }
    }
}

#[async_trait]
pub trait InstanceHost {
    async fn start_instance(&mut self) -> Result<Instance, Box<dyn std::error::Error>>;

    fn stop_instance(&self, instance: Instance);
}
