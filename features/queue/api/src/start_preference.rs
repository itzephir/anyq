#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StartPreference {
    #[default]
    Default,
    Start,
    Stop,
    Toggle,
}
