use clap::Args;

#[derive(Args)]
pub struct PluginRuntime {
    #[arg(value_enum)]
    pub mode: super::plugin_runtime_mode::PluginRuntimeMode,
}
