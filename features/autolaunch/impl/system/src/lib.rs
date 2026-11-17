mod native_registration;
pub use native_registration::NativeRegistration;

mod retained_executable;
pub use retained_executable::retain as retain_launcher;

#[cfg(any(target_os = "macos", target_os = "linux"))]
mod registration_file;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as platform;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as platform;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
use windows as platform;
