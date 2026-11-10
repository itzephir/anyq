#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterTaskRequest {
    pub name: String,
    pub topics: Vec<String>,
    pub start: crate::StartPreference,
}
