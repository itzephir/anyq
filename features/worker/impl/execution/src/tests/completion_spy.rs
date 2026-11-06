use super::{assignment, trace::Trace};
use anyq_queue_api::{CompleteDelivery, CompleteDeliveryRequest, Completion, Result};
pub struct CompletionSpy(pub Trace);
impl CompleteDelivery for CompletionSpy {
    fn complete(&mut self, request: CompleteDeliveryRequest) -> Result<Completion> {
        self.0.completions.borrow_mut().push(request);
        Ok(Completion {
            delivery: assignment().delivery,
            next: None,
        })
    }
}
