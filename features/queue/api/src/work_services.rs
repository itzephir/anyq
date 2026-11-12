use crate::{
    BeginCompletion, CompleteTaskOnce, ListManagedWork, RegisterManagedWork, ReportManagedWork,
    ResolveManagedWork,
};

pub struct WorkServices {
    pub cancel: Box<dyn crate::CancelManagedWork>,
    pub complete_once: Box<dyn CompleteTaskOnce>,
    pub begin_completion: Box<dyn BeginCompletion>,
    pub register: Box<dyn RegisterManagedWork>,
    pub report: Box<dyn ReportManagedWork>,
    pub list: Box<dyn ListManagedWork>,
    pub resolve: Box<dyn ResolveManagedWork>,
}
