//! Native daemon sessions route transport messages to narrow injected use cases.
mod queue_factory;
pub use queue_factory::QueueFactory;

mod session_signals;
pub use session_signals::SessionSignals;

mod route;
mod serve;
mod session;
pub use serve::serve;
