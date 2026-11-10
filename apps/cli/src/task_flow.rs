use crate::{
    checkpoint::checkpoint, error::Result, output::emit, presenters, wait_for_task::wait_for_task,
};
use anyq_queue_api::{CheckTaskRunning, CompleteTask, StartTask};
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
    start: &mut dyn CheckTaskRunning,
    completion: Option<&mut (dyn CompleteTask + 'service)>,
    cancelled: &AtomicBool,
) -> Result<()> {
    emit(&wait_for_task(name, start, cancelled)?, cancelled)?;

    if let Some(completion) = completion {
        emit(&complete(name, false, completion, cancelled)?, cancelled)?;
    }

    Ok(())
}

pub fn start_then_complete<'service>(
    name: &str,
    wait: bool,
    start: &mut dyn StartTask,
    completion: Option<&mut (dyn CompleteTask + 'service)>,
    cancelled: &AtomicBool,
) -> Result<()> {
    emit(
        &crate::start_task::start_task(name, wait, start, cancelled)?,
        cancelled,
    )?;

    if let Some(completion) = completion {
        emit(&complete(name, false, completion, cancelled)?, cancelled)?;
    }

    Ok(())
}
