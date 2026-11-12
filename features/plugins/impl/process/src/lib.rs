//! Explicit local plugin boundary.

mod native_process_factory;
pub use native_process_factory::NativeProcessFactory;

mod io_runtime;
mod process_handle;
mod supervise;
