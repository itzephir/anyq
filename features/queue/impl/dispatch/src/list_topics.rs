use crate::ports::*;
use anyq_queue_api::*;

pub struct ListTopicsInteractor<S> {
    store: S,
}
impl<S> ListTopicsInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}
impl<S: Store> ListTopics for ListTopicsInteractor<S> {
    fn list(&mut self) -> Result<Vec<Topic>> {
        self.store.transaction(|records| records.topics().topics())
    }
}
