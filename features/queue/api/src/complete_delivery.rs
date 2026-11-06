use crate::*;

pub trait CompleteDelivery {
    fn complete(&mut self, request: CompleteDeliveryRequest) -> Result<Completion>;
}
