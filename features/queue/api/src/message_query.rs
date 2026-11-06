#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageQuery {
    pub topic: Option<String>,
    pub group: Option<String>,
    pub after: i64,
    pub limit: u32,
}
