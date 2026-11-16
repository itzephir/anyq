use anyq_queue_api::{Error, Result};

pub const MAX_ENABLED_PLUGINS: usize = 16;

pub fn validate_catalog_name(name: &str) -> Result<()> {
    crate::validate_plugin_name(name)?;

    if name.starts_with('.') {
        return Err(Error::Invalid(
            "plugin names cannot begin with a dot".into(),
        ));
    }

    Ok(())
}
