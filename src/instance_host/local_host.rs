use crate::instance_host::{Instance, InstanceHost};

use async_trait::async_trait;
use std::process::{Command, Child};
use tracing::info;
use std::collections::HashMap;

pub struct LocalHost {
    processes: HashMap<String, Child>
}

impl LocalHost {
    pub fn new() -> LocalHost {
        LocalHost {
            processes: HashMap::new()
        }
    }
}

#[async_trait]
impl InstanceHost for LocalHost {
    async fn start_instance(
        &mut self,
        username: String,
    ) -> Result<Instance, Box<dyn std::error::Error>> {
        // TODO: argument providing path to python entrypoint, execute
        let child = Command::new("sh")
                .arg("-c")
                .arg("echo hello && sleep 30")
                .spawn()
                .expect("failed to execute process");
        self.processes.insert(username.clone(), child);
        let instance = Instance::new("0.0.0.0".to_string(), 8555);
        Ok(instance)
    }

    async fn stop_instance(&mut self, username: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut child = self.processes.get_mut(&username).expect("No process running");
        child.kill().expect("cannot kill"); // TODO: change it to graceful exit, then kill if cannot exit gracefully
        Ok(())
    }
}
