//! Atomic panel-local preference storage.
mod codec;
mod file_preferences;
pub use file_preferences::FilePreferences;

#[cfg(test)]
mod tests;
