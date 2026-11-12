use crate::Store;
use anyq_plugin_api::*;
use anyq_queue_api::{Error, Result};

pub struct EnablePluginInteractor<S> {
    store: S,
}

impl<S> EnablePluginInteractor<S> {
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S: Store> EnablePlugin for EnablePluginInteractor<S> {
    fn enable(&mut self, request: EnablePluginRequest) -> Result<PluginStatus> {
        validate_plugin_name(&request.name)?;
        if request.executable.is_empty()
            || request.executable.len() > 32768
            || request.arguments.len() > 128
            || request.arguments.iter().map(String::len).sum::<usize>() > 65536
        {
            return Err(Error::Invalid("invalid or oversized plugin command".into()));
        }
        self.store.transaction(|records| {
            let existing = records.registration(&request.name)?;
            if !existing.as_ref().is_some_and(|plugin| plugin.enabled)
                && records
                    .registrations()?
                    .iter()
                    .filter(|plugin| plugin.enabled)
                    .count()
                    >= 16
            {
                return Err(Error::Conflict("enabled plugin limit reached (16)".into()));
            }
            let mut registration = if let Some(existing) = records.registration(&request.name)? {
                if existing.executable != request.executable
                    || existing.arguments != request.arguments
                {
                    return Err(Error::Conflict(
                        "registered plugin command cannot change; use a new plugin name".into(),
                    ));
                }

                existing
            } else {
                PluginRegistration {
                    name: request.name,
                    executable: request.executable,
                    arguments: request.arguments,
                    enabled: false,
                    running: false,
                    cursor: records.watermark()?,
                    failures: 0,
                    retry_at: 0,
                    last_failure: None,
                    subscriptions: Vec::new(),
                    capabilities: Vec::new(),
                    state: Default::default(),
                }
            };
            registration.enabled = true;
            registration.retry_at = 0;
            registration.failures = 0;
            registration.last_failure = None;
            records.save(&registration)?;

            crate::read_status::read_status(records, &registration.name)
        })
    }
}
