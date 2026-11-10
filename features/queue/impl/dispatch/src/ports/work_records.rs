use anyq_queue_api::{CompletionReceipt, ManagedWork, ManagedWorkFilter, Result};

pub trait WorkRecords {
    /// Return matching rows in owner/action order, applying the exact page bound.
    fn managed(&mut self, filter: &ManagedWorkFilter) -> Result<Vec<ManagedWork>>;

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
