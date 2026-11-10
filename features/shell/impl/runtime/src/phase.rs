#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    Registering,
    Reporting,
    Executing,
    Completing,
    Finished,
    Uncertain,
}
