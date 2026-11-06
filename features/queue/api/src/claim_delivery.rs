use crate::*;

pub trait ClaimDelivery {
    fn claim(&mut self, request: Claim) -> Result<Option<Assignment>>;
}
