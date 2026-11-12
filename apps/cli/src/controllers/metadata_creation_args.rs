use clap::{ArgAction, Args};
use std::path::PathBuf;

#[derive(Args)]
pub struct MetadataCreationArgs {
    #[arg(long = "meta", num_args = 2, value_names = ["KEY", "VALUE"], action = ArgAction::Append, allow_hyphen_values = true)]
    pub patches: Vec<String>,
    #[arg(long, value_name = "FILE")]
    pub metafile: Option<PathBuf>,
}
