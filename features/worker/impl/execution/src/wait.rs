pub trait Wait {
    fn wait_ms(&mut self, duration: u64);

    fn stopped(&self) -> bool;
}
