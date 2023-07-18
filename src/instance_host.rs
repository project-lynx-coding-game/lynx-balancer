pub trait InstanceHost {
    fn start_instance() -> (String, u16);
    fn stop_instance(url: String, port: u16);
}
