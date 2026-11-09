use clap::Subcommand;

#[derive(Subcommand)]
pub enum ConfigAction {
    #[command(about = "List effective configuration values")]
    List,
}
