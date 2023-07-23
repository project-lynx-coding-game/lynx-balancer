use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Instance {
    url: String,
    port: u16
}

pub struct InstanceHost {}

impl InstanceHost {

    pub fn new() -> InstanceHost {
        InstanceHost{}
    }

    pub fn start_instance(&self) -> Instance {
        todo!()
    }

    pub fn stop_instance(&self, instance: Instance) {
        println!("{:?}", instance);
        todo!()
    }
}
