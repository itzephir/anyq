use anyq_queue_api::Result;

pub(crate) fn checkpoint(_name: &str) -> Result<()> {
    #[cfg(feature = "integration-test-hooks")]
    {
        use anyq_queue_api::Error;
        use std::{env, fs, path::PathBuf, thread, time::Duration};
        if env::var("ANYQ_SHELL_TEST_CHECKPOINT").ok().as_deref() == Some(_name) {
            let directory = PathBuf::from(
                env::var_os("ANYQ_SHELL_TEST_DIRECTORY")
                    .ok_or_else(|| Error::Invalid("shell fixture directory is missing".into()))?,
            );
            fs::create_dir_all(&directory).map_err(|error| Error::Storage(error.to_string()))?;
            fs::write(directory.join("ready"), _name)
                .map_err(|error| Error::Storage(error.to_string()))?;
            while !directory.join("release").exists() {
                if directory.join("crash").exists() {
                    std::process::exit(88);
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
    }

    Ok(())
}
