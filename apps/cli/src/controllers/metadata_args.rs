use clap::Args;

#[derive(Args)]
pub struct MetadataArgs {
    pub name: String,
    pub key: Option<String>,
    #[arg(requires = "key", allow_hyphen_values = true)]
    pub value: Option<String>,
    #[arg(long, value_name = "KEY", conflicts_with_all = ["key", "value"])]
    pub unset: Option<String>,
}
