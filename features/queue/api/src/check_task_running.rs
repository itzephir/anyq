use crate::Result;

pub trait CheckTaskRunning {
    fn check_running(&mut self, name: &str) -> Result<bool>;
}
