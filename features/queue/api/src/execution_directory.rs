/// Native absolute caller directory, captured by the outer application boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionDirectory {
    Unix(Vec<u8>),
    Windows(Vec<u16>),
}
