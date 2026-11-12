use crate::ports::Store;
use anyq_queue_api::*;

pub struct CompleteTaskOnceInteractor<S> {
    store: S,
}

impl<S> CompleteTaskOnceInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> CompleteTaskOnce for CompleteTaskOnceInteractor<S> {
    fn complete(&mut self, request: CompleteTaskOnceRequest) -> Result<Task> {
        self.store.transaction(|records| {
            validate_action_identity(&request.owner, &request.action_id)?;
            let task = parse_name(&request.task)?;
            if records
                .work()
                .managed(&ManagedWorkFilter {
                    owner: Some(request.owner.clone()),
                    action: Some(request.action_id.clone()),
                    ..Default::default()
                })?
                .iter()
                .any(|work| work.action_id == request.action_id)
            {
                return Err(Error::Conflict(
                    "action ID is already used for managed execution".into(),
                ));
            }
            if let Some(receipt) = records.work().receipt(&request.owner, &request.action_id)? {
                if receipt.task != task || receipt.force != request.force {
                    return Err(Error::Conflict(
                        "action ID reused with a different completion request".into(),
                    ));
                }

                return Ok(receipt.result);
            }

            let result = crate::complete_task::complete_in_transaction(
                records,
                &format_name(&task),
                request.force,
                QueuePolicy::default(),
            )?;
            records.work().save_receipt(
                &request.owner,
                &request.action_id,
                &CompletionReceipt {
                    task,
                    force: request.force,
                    result: result.clone(),
                },
            )?;

            Ok(result)
        })
    }
}
