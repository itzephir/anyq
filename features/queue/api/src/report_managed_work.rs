use crate::{ManagedWork, ReportManagedWorkRequest, Result};

pub trait ReportManagedWork {
    fn report(&mut self, request: ReportManagedWorkRequest) -> Result<ManagedWork>;
}
