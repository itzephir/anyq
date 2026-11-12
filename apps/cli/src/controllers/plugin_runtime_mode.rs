use clap::ValueEnum;

#[derive(Clone, ValueEnum)]
pub enum PluginRuntimeMode {
    Timeout,
    Shell,
    #[value(hide = true)]
    ShellAnchor,
}
