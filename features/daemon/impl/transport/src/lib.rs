//! Native private endpoints, lifetime locking and cancellable bounded framing.
mod endpoint;
pub use endpoint::{Endpoint, validate_peer};

mod owner_lock;
pub use owner_lock::OwnerLock;

mod frame_reader;
pub use frame_reader::FrameReader;

mod send_frame;
pub use send_frame::send_frame;

pub use interprocess::local_socket::{Listener, Stream, traits};
