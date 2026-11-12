use crate::EntryOutput;

pub struct EntryOutcome {
    pub code: Option<i32>,
    pub success: bool,
    pub output: EntryOutput,
}
