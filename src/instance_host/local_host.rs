use crate::instance_host::{Instance, InstanceHost};

use async_trait::async_trait;
use std::collections::HashMap;
use std::net::TcpListener;
use std::process::{Child, Command};


pub struct LocalHost {
    processes: HashMap<String, Child>,
    app_directory: String, // we assume it is a FastAPI app (lynx-scene-host), uvicorn required
}

impl LocalHost {
    pub fn new(app_directory: String) -> LocalHost {
        LocalHost {
            processes: HashMap::new(),
            app_directory,
        }
    }
}

fn get_available_port() -> Option<u16> {
    (8000..9000).find(|port| port_is_available(*port))
}

fn port_is_available(port: u16) -> bool {
    match TcpListener::bind(("0.0.0.0", port)) {
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
        let port = get_available_port().expect("no available ports");
        let child = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "cd {} && uvicorn main:app --port {} --host 0.0.0.0",
                self.app_directory, port
            ))
            .spawn()
            .expect("failed to execute process");
        self.processes.insert(username.clone(), child);
        let instance = Instance::new("0.0.0.0".to_string(), port);
        Ok(instance)
    }

    async fn stop_instance(&mut self, username: String) -> Result<(), Box<dyn std::error::Error>> {
        let child = self
            .processes
            .get_mut(&username)
            .expect("No process running");
        child.kill().expect("cannot kill"); // TODO: change it to graceful exit, then kill if cannot exit gracefully
        Ok(())
    }
}
