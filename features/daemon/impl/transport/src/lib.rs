//! Native private endpoints, lifetime locking and cancellable bounded framing.
const MORE: u32 = 1 << 31;

mod endpoint;
pub use endpoint::{Endpoint, validate_peer};

mod owner_lock;
pub use owner_lock::OwnerLock;

mod decode_json;
mod frame_reader;
mod receive_deadline;
pub use frame_reader::FrameReader;

#[cfg(windows)]
mod windows_peer;

mod send_frame;
pub use send_frame::{send_bounded_frame, send_frame};

mod io_runtime;
mod stream;
pub use stream::Stream;

mod listener;
pub use listener::Listener;

#[cfg(test)]
mod tests;
