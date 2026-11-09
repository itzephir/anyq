use crate::Result;

pub trait ListTasks {
    fn list(&mut self) -> Result<Vec<String>>;
}
