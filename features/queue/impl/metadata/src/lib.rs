//! Outer format conversion shared by SQLite and the CLI composition root.
mod json;
mod json_object;
mod write_json;

pub use json::{parse_json, parse_value};
pub use write_json::to_json;

#[cfg(feature = "files")]
mod yaml_input;

#[cfg(feature = "files")]
mod yaml_scalar;

#[cfg(feature = "files")]
pub fn parse_yaml(input: &str) -> anyq_queue_api::Result<anyq_queue_api::Metadata> {
    yaml_input::YamlInput::parse(input)
}

#[cfg(feature = "files")]
mod toml;

#[cfg(feature = "files")]
pub use toml::parse_toml;

#[cfg(test)]
mod tests;
