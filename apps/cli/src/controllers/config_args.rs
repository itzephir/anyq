use clap::Args;

#[derive(Args)]
pub struct ConfigArgs {
    #[arg(short = 'l', long, conflicts_with_all = ["get", "unset", "words"])]
    pub list: bool,
    #[arg(long, conflicts_with = "unset")]
    pub get: bool,
    #[arg(long)]
    pub unset: bool,
    #[arg(num_args = 0..=2, value_name = "KEY [VALUE]")]
    pub words: Vec<String>,
}
