use anyq_queue_api::{ExecutionDirectory, Result};
use anyq_shell_api::{Plan, PreparedPlan};

pub trait PreparePlan {
    fn prepare(&mut self, plan: Plan, origin: Option<ExecutionDirectory>) -> Result<PreparedPlan>;
}
