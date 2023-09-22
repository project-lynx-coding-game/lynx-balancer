use crate::instance_host::{Instance, InstanceHost};

use async_trait::async_trait;
use std::process::Command;

use tracing::info;

pub struct LocalHost {}

impl LocalHost {
    pub fn new() -> LocalHost {
        LocalHost {}
    }
}

#[async_trait]
impl InstanceHost for LocalHost {
    async fn start_instance(
        &mut self,
        username: String,
    ) -> Result<Instance, Box<dyn std::error::Error>> {
        Command::new("sh")
                .arg("-c")
                .arg("echo hello")
                .output()
                .expect("failed to execute process");
        let instance = Instance::new("127.0.0.1".to_string(), 8555);
        Ok(instance)
    }

    async fn stop_instance(&self, username: String) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
