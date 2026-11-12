use crate::PluginTimer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimerUpdate {
    Put(PluginTimer),
    Remove(String),
}
