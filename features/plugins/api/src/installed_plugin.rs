#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledPlugin {
    pub name: String,
    pub executable: String,
    pub available: bool,
    pub enabled: bool,
    pub revision: u64,
}
