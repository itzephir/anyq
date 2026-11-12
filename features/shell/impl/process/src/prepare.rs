use anyq_queue_api::{Error, ExecutionDirectory, Result};
use anyq_shell_api::{Command, Plan, PreparedPlan, Source};
use std::path::{Path, PathBuf};

pub(crate) fn directory(value: &ExecutionDirectory) -> Result<PathBuf> {
    #[cfg(unix)]
    if let ExecutionDirectory::Unix(bytes) = value {
        use std::os::unix::ffi::OsStringExt;
        return Ok(std::ffi::OsString::from_vec(bytes.clone()).into());
    }
    #[cfg(windows)]
    if let ExecutionDirectory::Windows(units) = value {
        use std::os::windows::ffi::OsStringExt;
        return Ok(std::ffi::OsString::from_wide(units).into());
    }

    Err(Error::Invalid(
        "execution directory belongs to another operating system".into(),
    ))
}

fn native_directory(value: &Path) -> ExecutionDirectory {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        ExecutionDirectory::Unix(value.as_os_str().as_bytes().to_vec())
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        ExecutionDirectory::Windows(value.as_os_str().encode_wide().collect())
    }
}

pub(crate) fn prepare(plan: Plan, origin: Option<ExecutionDirectory>) -> Result<PreparedPlan> {
    let cwd = match plan.cwd.as_ref() {
        Some(cwd) => {
            let path = PathBuf::from(cwd);
            if !path.is_absolute() {
                return Err(Error::Invalid("shell.cwd must be absolute".into()));
            }
            path
        }
        None => directory(&origin.ok_or_else(|| {
            Error::Invalid(
                "task has no captured origin; set absolute shell.cwd before starting".into(),
            )
        })?)?,
    };
    if !cwd.is_absolute() {
        return Err(Error::Invalid(
            "captured shell directory must be absolute".into(),
        ));
    }
    let cwd = native_directory(&cwd);
    let default_interpreter = {
        #[cfg(unix)]
        {
            vec!["/bin/sh".into(), "-c".into()]
        }
        #[cfg(windows)]
        {
            let system = std::env::var_os("SystemRoot").ok_or_else(|| {
                Error::Invalid("SystemRoot is required for the native cmd interpreter".into())
            })?;
            vec![
                PathBuf::from(system)
                    .join("System32")
                    .join("cmd.exe")
                    .to_string_lossy()
                    .into_owned(),
                "/D".into(),
                "/S".into(),
                "/C".into(),
            ]
        }
    };
    let interpreter = plan.interpreter.clone().unwrap_or(default_interpreter);
    let mut commands = Vec::new();
    for entry in &plan.entries {
        let (program, arguments, raw_shell) = if plan.source == Source::Shell {
            #[cfg(unix)]
            let (args, raw) = {
                let mut args = interpreter[1..].to_vec();
                args.push(entry.clone());
                (args, None)
            };
            #[cfg(windows)]
            let (args, raw) = (interpreter[1..].to_vec(), Some(entry.clone()));
            (interpreter[0].clone(), args, raw)
        } else {
            let path = PathBuf::from(entry);
            let path = if path.is_absolute() {
                path
            } else {
                directory(&cwd)?.join(path)
            };
            let text = path
                .to_str()
                .ok_or_else(|| {
                    Error::Invalid("src metadata must resolve to a Unicode executable path".into())
                })?
                .to_string();
            if plan.interpreter.is_some() {
                let mut args = interpreter[1..].to_vec();
                args.push(text);
                (interpreter[0].clone(), args, None)
            } else {
                (text, Vec::new(), None)
            }
        };
        commands.push(Command {
            program,
            arguments,
            raw_shell,
            directory: cwd.clone(),
            environment: plan.environment.clone(),
        });
    }

    Ok(PreparedPlan {
        commands,
        autocomplete: plan.autocomplete,
    })
}
