use std::path::PathBuf;

/// External test fixture only; never included in an ordinary binary or protocol.
pub(crate) struct FixtureClock {
    path: PathBuf,
}

impl FixtureClock {
    pub fn from_environment() -> Option<Self> {
        std::env::var_os("ANYQ_TEST_CLOCK_FILE").map(|path| Self { path: path.into() })
    }

    fn millis(&self) -> i64 {
        std::fs::read_to_string(&self.path)
            .expect("read isolated clock fixture")
            .trim()
            .parse()
            .expect("integer clock fixture")
    }
}

impl anyq_queue_dispatch::ports::Clock for FixtureClock {
    fn now_millis(&self) -> i64 {
        self.millis()
    }
}

impl anyq_plugin_runtime::Clock for FixtureClock {
    fn now_millis(&self) -> i64 {
        self.millis()
    }
}
