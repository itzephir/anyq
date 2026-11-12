mod deadline;
mod duration;
mod resolve;

pub use deadline::Deadline;
pub use duration::Duration;
pub use resolve::resolve;

#[cfg(test)]
mod tests;
