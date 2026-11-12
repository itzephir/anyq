use clap::Args;

#[derive(Args)]
pub struct PluginArgs {
    #[command(subcommand)]
    pub command: super::PluginCommand,
}
