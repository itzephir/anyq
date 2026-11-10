mod drain;
mod native_entry;
mod native_factory;
mod prepare;
#[cfg(unix)]
mod unix_child;
#[cfg(windows)]
mod windows_child;

pub use native_factory::NativeFactory;
#[cfg(unix)]
use unix_child::UnixChild as NativeChild;
#[cfg(windows)]
use windows_child::WindowsChild as NativeChild;
