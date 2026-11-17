use std::{
    thread,
    time::{Duration, Instant},
};

#[track_caller]
pub fn eventually(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while !condition() {
        assert!(
            Instant::now() < deadline,
            "observable test condition timed out"
        );
        thread::sleep(Duration::from_millis(10));
    }
}
