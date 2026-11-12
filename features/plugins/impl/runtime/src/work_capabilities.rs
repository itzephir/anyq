use anyq_queue_api::{
    BeginCompletion, CancelManagedWork, CompleteTaskOnce, ListManagedWork, RegisterManagedWork,
    ReportManagedWork,
};

/// The executable host has no operator cleanup-resolution capability.
pub struct WorkCapabilities {
    pub complete_once: Box<dyn CompleteTaskOnce>,
    pub begin_completion: Box<dyn BeginCompletion>,
    pub register: Box<dyn RegisterManagedWork>,
    pub report: Box<dyn ReportManagedWork>,
    pub list: Box<dyn ListManagedWork>,
    pub cancel: Box<dyn CancelManagedWork>,
}
