//! Native daemon sessions route transport messages to narrow injected use cases.
mod queue_factory;
pub use queue_factory::QueueFactory;

mod session_signals;
pub use session_signals::SessionSignals;

mod route;
mod serve;
mod session;
mod session_capacity;
mod session_class;
mod session_permit;
pub use serve::serve;
