use super::*;

pub trait UnitOfWork {
    fn topics(&mut self) -> &mut dyn TopicRecords;
    fn groups(&mut self) -> &mut dyn GroupRecords;
    fn messages(&mut self) -> &mut dyn MessageRecords;
    fn deliveries(&mut self) -> &mut dyn DeliveryRecords;
}
