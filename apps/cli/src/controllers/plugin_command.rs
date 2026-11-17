use clap::Subcommand;

#[derive(Subcommand)]
pub enum PluginCommand {
    #[command(about = "Enable an installed plugin for every active queue")]
    Enable {
        name: String,
    },
    #[command(about = "List installed plugins and user-wide enablement")]
    List,
    Status {
        name: String,
    },
    #[command(about = "Explicitly inspect bounded plugin-owned metadata state")]
    State {
        name: String,
    },
    #[command(about = "Disable a plugin for every active queue")]
    Disable {
        name: String,
    },
    #[command(about = "Inspect managed actions and unresolved execution")]
    Inspect {
        name: String,
    },
    #[command(about = "Record confirmed external cleanup of one managed action")]
    Resolve {
        name: String,
        action_id: String,
        #[arg(long, required = true)]
        confirmed_cleanup: bool,
    },
}
