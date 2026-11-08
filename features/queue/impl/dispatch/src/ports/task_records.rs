use super::TaskFilter;
use anyq_queue_api::{Result, Task};

pub trait TaskRecords {
    fn task_names(&mut self) -> Result<Vec<String>>;

    fn tasks(&mut self, filter: &TaskFilter) -> Result<Vec<Task>>;

    fn task(&mut self, name: &str) -> Result<Option<Task>>;

    fn insert_task(&mut self, name: &str) -> Result<Task>;

    fn save_task(&mut self, task: &Task) -> Result<()>;

    fn count_tasks(&mut self, filter: &TaskFilter) -> Result<u64>;
}
