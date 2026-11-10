use clap::Subcommand;

#[derive(Subcommand)]
pub enum PluginCommand {
    #[command(about = "Explicitly trust and enable a local executable")]
    Enable {
        name: String,
        executable: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        arguments: Vec<String>,
    },
    List,
    Status {
        name: String,
    },
    #[command(about = "Explicitly inspect bounded plugin-owned metadata state")]
    State {
        name: String,
    },
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
