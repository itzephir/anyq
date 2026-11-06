use super::{assignment, trace::Trace};
use anyq_queue_api::{Delivery, RejectDelivery, RejectDeliveryRequest, Result};
pub struct RejectionSpy(pub Trace);
impl RejectDelivery for RejectionSpy {
    fn reject(&mut self, request: RejectDeliveryRequest) -> Result<Delivery> {
        self.0.rejections.borrow_mut().push(request);
        Ok(assignment().delivery)
    }
}
