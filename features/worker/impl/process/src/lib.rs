//! Native process adapters. No queue policies live here.
#[cfg(unix)]
mod unix;
#[cfg(unix)]
use unix as platform;

#[cfg(windows)]
#[allow(unsafe_code)]
mod windows;
#[cfg(windows)]
use windows as platform;

mod command_runner;
pub use command_runner::CommandRunner;

mod managed_process;
mod system_wait;
pub use system_wait::SystemWait;

fn io(error: std::io::Error) -> anyq_queue_api::Error {
    anyq_queue_api::Error::Storage(error.to_string())
}
