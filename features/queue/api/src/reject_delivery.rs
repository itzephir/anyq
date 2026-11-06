use crate::*;

pub trait RejectDelivery {
    fn reject(&mut self, request: RejectDeliveryRequest) -> Result<Delivery>;
}
