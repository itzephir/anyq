#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnablePluginRequest {
    pub name: String,
    pub executable: String,
    pub arguments: Vec<String>,
}
