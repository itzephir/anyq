use anyq_plugin_api::PluginServices;
use anyq_queue_api::{ListManagedWork, QueueServices, ResolveManagedWork};

/// Passive server session handles, assembled explicitly at the daemon root.
pub struct DaemonServices {
    pub queue: QueueServices,
    pub plugins: PluginServices,
    pub list_work: Box<dyn ListManagedWork>,
    pub resolve_work: Box<dyn ResolveManagedWork>,
}
