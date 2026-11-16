use super::autolaunch_state::AutolaunchState;
use clap::Args;

#[derive(Args)]
pub struct AutolaunchArgs {
    #[arg(
        value_enum,
        help = "Enable or disable launch at login; omit to read system status"
    )]
    pub state: Option<AutolaunchState>,
}
