use clap::ValueEnum;

#[derive(Clone, Copy, ValueEnum)]
pub enum AutolaunchState {
    On,
    Off,
}

impl AutolaunchState {
    pub fn enabled(self) -> bool {
        matches!(self, Self::On)
    }
}
