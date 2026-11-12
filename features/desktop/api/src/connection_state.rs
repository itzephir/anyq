#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ConnectionState {
    #[default]
    Loading,
    Live,
    Stale,
    Error,
}

impl ConnectionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Loading => "loading",
            Self::Live => "live",
            Self::Stale => "stale",
            Self::Error => "error",
        }
    }
}
