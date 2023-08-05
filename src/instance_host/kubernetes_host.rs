use crate::instance_host::{Instance, InstanceHost};

pub struct KubernetesHost {}

impl KubernetesHost {
    pub fn new() -> KubernetesHost {
        KubernetesHost {}
    }
}

impl InstanceHost for KubernetesHost {
    fn start_instance(&self) -> Instance {
        todo!()
    }

    fn stop_instance(&self, instance: Instance) {
        println!("{:?}", instance);
        todo!()
    }
}
