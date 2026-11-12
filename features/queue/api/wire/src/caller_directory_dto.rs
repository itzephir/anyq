use anyq_queue_api::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Preserve native path encoding rather than requiring a UTF-8 caller directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CallerDirectoryDto {
    #[cfg(unix)]
    Unix(Vec<u8>),
    #[cfg(windows)]
    Windows(Vec<u16>),
}

impl From<PathBuf> for CallerDirectoryDto {
    fn from(path: PathBuf) -> Self {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;

            Self::Unix(path.into_os_string().into_vec())
        }

        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;

            Self::Windows(path.as_os_str().encode_wide().collect())
        }
    }
}

impl TryFrom<CallerDirectoryDto> for PathBuf {
    type Error = Error;

    fn try_from(value: CallerDirectoryDto) -> Result<Self> {
        match value {
            #[cfg(unix)]
            CallerDirectoryDto::Unix(bytes) => {
                use std::os::unix::ffi::OsStringExt;

                if bytes.contains(&0) {
                    return Err(Error::Invalid("caller directory contains NUL".into()));
                }

                Ok(std::ffi::OsString::from_vec(bytes).into())
            }
            #[cfg(windows)]
            CallerDirectoryDto::Windows(wide) => {
                use std::os::windows::ffi::OsStringExt;

                if wide.contains(&0) {
                    return Err(Error::Invalid("caller directory contains NUL".into()));
                }

                Ok(std::ffi::OsString::from_wide(&wide).into())
            }
        }
    }
}

impl From<CallerDirectoryDto> for anyq_queue_api::ExecutionDirectory {
    fn from(value: CallerDirectoryDto) -> Self {
        match value {
            #[cfg(unix)]
            CallerDirectoryDto::Unix(bytes) => Self::Unix(bytes),
            #[cfg(windows)]
            CallerDirectoryDto::Windows(wide) => Self::Windows(wide),
        }
    }
}

impl TryFrom<anyq_queue_api::ExecutionDirectory> for CallerDirectoryDto {
    type Error = Error;

    fn try_from(value: anyq_queue_api::ExecutionDirectory) -> Result<Self> {
        match value {
            #[cfg(unix)]
            anyq_queue_api::ExecutionDirectory::Unix(bytes) => Ok(Self::Unix(bytes)),
            #[cfg(windows)]
            anyq_queue_api::ExecutionDirectory::Windows(wide) => Ok(Self::Windows(wide)),
            _ => Err(Error::Invalid(
                "caller directory belongs to another platform".into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CallerDirectoryDto, PathBuf};

    #[cfg(unix)]
    #[test]
    fn native_path_bytes_round_trip_without_utf8_conversion() {
        use std::os::unix::ffi::OsStringExt;

        let path = PathBuf::from(std::ffi::OsString::from_vec(vec![b'/', 0xff]));

        assert_eq!(
            PathBuf::try_from(CallerDirectoryDto::from(path.clone())).unwrap(),
            path
        );
        assert!(PathBuf::try_from(CallerDirectoryDto::Unix(vec![0])).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn native_wide_paths_round_trip_without_utf8_conversion() {
        use std::os::windows::ffi::OsStringExt;

        let path = PathBuf::from(std::ffi::OsString::from_wide(&[67, 58, 92, 0xd800]));

        assert_eq!(
            PathBuf::try_from(CallerDirectoryDto::from(path.clone())).unwrap(),
            path
        );
        assert!(PathBuf::try_from(CallerDirectoryDto::Windows(vec![0])).is_err());
    }
}
