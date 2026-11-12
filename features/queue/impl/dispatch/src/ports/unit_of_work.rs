use super::{Clock, EventRecords, TaskRecords, TopicRecords, WorkRecords};
use anyq_queue_api::ExecutionDirectory;

pub trait UnitOfWork {
    fn work(&mut self) -> &mut dyn WorkRecords;

    fn clock(&self) -> &dyn Clock;

    fn caller_directory(&self) -> Option<&ExecutionDirectory>;

    fn events(&mut self) -> &mut dyn EventRecords;

    fn topics(&mut self) -> &mut dyn TopicRecords;

    fn tasks(&mut self) -> &mut dyn TaskRecords;
}
