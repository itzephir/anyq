use super::{TaskRecords, TopicRecords};

pub trait UnitOfWork {
    fn topics(&mut self) -> &mut dyn TopicRecords;

    fn tasks(&mut self) -> &mut dyn TaskRecords;
}
