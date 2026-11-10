use clap::Subcommand;

#[derive(Subcommand)]
pub enum DaemonCommand {
    #[command(about = "Start or reuse the background queue owner")]
    Start,
    #[command(about = "Serve the queue in the foreground")]
    Run,
    #[command(about = "Inspect the owner without starting it")]
    Status,
    #[command(about = "Stop the owner without releasing task resources")]
    Stop,
}
