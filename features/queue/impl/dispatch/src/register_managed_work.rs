use crate::ports::Store;
use anyq_queue_api::*;

pub struct RegisterManagedWorkInteractor<S> {
    store: S,
}

impl<S> RegisterManagedWorkInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> RegisterManagedWork for RegisterManagedWorkInteractor<S> {
    fn register(&mut self, request: RegisterManagedWorkRequest) -> Result<ManagedLaunch> {
        self.store.transaction(|records| {
            validate_action_identity(&request.owner, &request.action_id)?;
            if records
                .work()
                .receipt(&request.owner, &request.action_id)?
                .is_some()
            {
                return Err(Error::Conflict(
                    "action ID is already used for queue completion".into(),
                ));
            }
            let task =
                crate::lookup_task::lookup_task(records, &request.task, QueuePolicy::default())?;
            if let Some(work) = records
                .work()
                .managed(Some(&request.owner), None)?
                .into_iter()
                .find(|work| work.action_id == request.action_id)
            {
                if work.task != task.name() {
                    return Err(Error::Conflict(
                        "managed action ID reused for another task".into(),
                    ));
                }

                return Ok(ManagedLaunch {
                    work,
                    launch_allowed: false,
                });
            }
            let completion = records.work().completion_requested(task.name())?;
            ManagedWork::require_launch(task.state(), completion)?;
            let work = ManagedWork {
                owner: request.owner,
                action_id: request.action_id,
                task: task.name().into(),
                state: ManagedWorkState::LaunchPending,
                cancellation_requested: false,
            };
            records.work().save_managed(&work)?;

            Ok(ManagedLaunch {
                work,
                launch_allowed: true,
            })
        })
    }
}
