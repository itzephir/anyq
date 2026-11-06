use super::json::json;
use anyq_queue_api::{Error, Result};
use clap::Args;
use std::{io::Read, path::PathBuf};

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct Data {
    #[arg(long, help = "JSON payload")]
    pub payload: Option<String>,
    #[arg(long, help = "Read JSON from a file; '-' reads stdin")]
    pub file: Option<PathBuf>,
}

impl Data {
    pub fn read(self) -> Result<String> {
        let raw = if let Some(s) = self.payload {
            s
        } else {
            let file = self
                .file
                .ok_or_else(|| Error::Invalid("payload or file required".into()))?;
            let mut source: Box<dyn Read> = if file.as_os_str() == "-" {
                Box::new(std::io::stdin())
            } else {
                Box::new(std::fs::File::open(file).map_err(|e| Error::Invalid(e.to_string()))?)
            };

            let mut raw = String::new();
            source
                .by_ref()
                .take(1_048_577)
                .read_to_string(&mut raw)
                .map_err(|e| Error::Invalid(e.to_string()))?;
            raw
        };

        if raw.len() > 1_048_576 {
            return Err(Error::Invalid(
                "payload exceeds 1 MiB; put large artifacts in files and publish paths".into(),
            ));
        }

        json(&raw)
    }
}
