use clap::Subcommand;

#[derive(Subcommand)]
pub enum Groups {
    Create { topic: String, name: String },
    List { topic: String },
}
