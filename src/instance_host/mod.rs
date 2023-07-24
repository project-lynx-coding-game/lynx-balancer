pub mod kubernetes_host;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Instance {
    url: String,
    port: u16,
}

pub trait InstanceHost {
    fn start_instance(&self) -> Instance;

    fn stop_instance(&self, instance: Instance);
}
