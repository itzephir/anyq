#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SessionClass {
    Command,
    Waiting,
    Control,
}
