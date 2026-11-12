pub trait Clock {
    fn now_millis(&self) -> i64;
}
