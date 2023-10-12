use crate::instance_host::{Instance, InstanceHost};

use async_trait::async_trait;
use std::net::TcpListener;
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

fn get_available_port() -> Option<u16> {
    (8000..9000)
        .find(|port| port_is_available(*port))
}

fn port_is_available(port: u16) -> bool {
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[async_trait]
impl InstanceHost for LocalHost {
    async fn start_instance(
        &mut self,
        username: String,
    ) -> Result<Instance, Box<dyn std::error::Error>> {
        // TODO: if existing user, first kill previous
        let port = get_available_port().expect("no available ports");
        let child = Command::new("sh")
                .arg("-c")
                .arg(format!("podman run -p {}:{} ghcr.io/group-project-gut/lynx-scene-host-python:latest main:app --port {} --host 0.0.0.0 --workers 1", port, port, port))
                .spawn()
                .expect("failed to execute process");
        self.processes.insert(username.clone(), child);
        let instance = Instance::new("0.0.0.0".to_string(), port);
        Ok(instance)
    }

    async fn stop_instance(&mut self, username: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut child = self.processes.get_mut(&username).expect("No process running");
        child.kill().expect("cannot kill"); // TODO: change it to graceful exit, then kill if cannot exit gracefully
        Ok(())
    }
}
