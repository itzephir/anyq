use anyq_config_api::{DatabaseSelection, DatabaseSource, Error, Result};
use std::{env, path::PathBuf};

pub fn home() -> Result<PathBuf> {
    #[cfg(windows)]
    let path = env::var_os("USERPROFILE").or_else(|| env::var_os("HOME"));
    #[cfg(not(windows))]
    let path = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE"));

    path.map(PathBuf::from)
        .ok_or_else(|| Error::Invalid("home directory is unavailable".into()))
}

pub fn default_db() -> Result<PathBuf> {
    #[cfg(windows)]
    let data = env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| {
            Error::Invalid("set ANYQ_DB, db.path or --db: LOCALAPPDATA is unavailable".into())
        })?;
    #[cfg(not(windows))]
    let data = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|path| PathBuf::from(path).join(".local/share")))
        .ok_or_else(|| {
            Error::Invalid(
                "set ANYQ_DB, db.path or --db: HOME and XDG_DATA_HOME are unavailable".into(),
            )
        })?;

    Ok(data.join("anyq/queue.db"))
}

pub fn configured_db(value: &str) -> Result<PathBuf> {
    if value == "~" {
        return home();
    }

    if let Some(relative) = value
        .strip_prefix("~/")
        .or_else(|| value.strip_prefix("~\\"))
    {
        return Ok(home()?.join(relative));
    }

    Ok(PathBuf::from(value))
}

pub fn resolve_database(
    explicit: Option<&std::path::Path>,
    configured: Option<&str>,
) -> Result<DatabaseSelection> {
    let environment = env::var_os("ANYQ_DB");
    let source = DatabaseSource::select(
        explicit.is_some(),
        environment.is_some(),
        configured.is_some(),
    );
    let path = match source {
        DatabaseSource::Override => explicit.expect("selected explicit path").to_owned(),
        DatabaseSource::Environment => environment.expect("selected environment path").into(),
        DatabaseSource::Configuration => {
            configured_db(configured.expect("selected configuration path"))?
        }
        DatabaseSource::Default => default_db()?,
    };

    Ok(DatabaseSelection { path, source })
}
