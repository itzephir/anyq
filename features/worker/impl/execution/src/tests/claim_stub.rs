use anyq_queue_api::{Assignment, Claim, ClaimDelivery, Result};

pub struct ClaimStub(pub Option<Assignment>);

impl ClaimDelivery for ClaimStub {
    fn claim(&mut self, _: Claim) -> Result<Option<Assignment>> {
        Ok(self.0.take())
    }
}
