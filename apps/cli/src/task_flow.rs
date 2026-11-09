use crate::{
    checkpoint::checkpoint, error::Result, output::emit, presenters, wait_for_task::wait_for_task,
};
use anyq_queue_api::{CompleteTask, TryStartTask};
use std::sync::atomic::AtomicBool;

pub fn complete(
    name: &str,
    force: bool,
    completion: &mut dyn CompleteTask,
    cancelled: &AtomicBool,
) -> Result<String> {
    checkpoint("cli.before-completion", cancelled)?;
    let task = completion.complete(name, force)?;
    checkpoint("cli.after-completion", cancelled)?;

    Ok(presenters::completed(task.name()))
}

pub fn wait_then_complete<'service>(
    name: &str,
    start: &mut dyn TryStartTask,
    completion: Option<&mut (dyn CompleteTask + 'service)>,
    cancelled: &AtomicBool,
) -> Result<()> {
    emit(&wait_for_task(name, start, cancelled)?)?;

    if let Some(completion) = completion {
        emit(&complete(name, false, completion, cancelled)?)?;
    }

    Ok(())
}
