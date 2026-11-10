use anyq_queue_api::Result;

pub trait PluginProcess {
    fn send(&mut self, frame: Vec<u8>) -> Result<()>;

    fn receive(&mut self) -> Result<Option<Vec<u8>>>;

    fn stop(&mut self) -> Result<()>;
}
