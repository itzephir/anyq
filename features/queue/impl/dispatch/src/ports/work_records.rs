use anyq_queue_api::{CompletionReceipt, ManagedWork, Result};

pub trait WorkRecords {
    fn managed(&mut self, owner: Option<&str>, task: Option<&str>) -> Result<Vec<ManagedWork>>;

    fn save_managed(&mut self, work: &ManagedWork) -> Result<()>;

    fn completion_requested(&mut self, task: &str) -> Result<bool>;

    fn request_completion(&mut self, task: &str) -> Result<()>;

    fn receipt(&mut self, owner: &str, action_id: &str) -> Result<Option<CompletionReceipt>>;

    fn save_receipt(
        &mut self,
        owner: &str,
        action_id: &str,
        receipt: &CompletionReceipt,
    ) -> Result<()>;
}
