use crate::{
    Clock, HostOptions, ProcessFactory, handle_frame::handle_frame, plugin_session::PluginSession,
    schedule::schedule,
};
use anyq_plugin_api::{PluginRegistration, retry_delay};
use anyq_plugin_dispatch::Store;
use anyq_plugin_wire::{HostFrame, MAX_FRAME_BYTES, PROTOCOL_VERSION, PluginFrame, decode, encode};
use anyq_queue_api::*;
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

pub struct Host<S> {
    pub store: S,
    pub events: Box<dyn ReadQueueEvents>,
    pub read: Box<dyn GetTask>,
    pub work: crate::WorkCapabilities,
    pub clock: Box<dyn Clock>,
    pub processes: Box<dyn ProcessFactory>,
    pub options: HostOptions,
}

impl<S: Store> Host<S> {
    fn unresolved(&mut self, name: &str) -> Result<Vec<ManagedWork>> {
        Ok(self
            .work
            .list
            .list(ListManagedWorkRequest {
                owner: Some(name.into()),
                task: None,
            })?
            .into_iter()
            .filter(|work| work.state.blocks_completion())
            .collect())
    }

    fn uncertain(&mut self, name: &str) -> Result<()> {
        for work in self.unresolved(name)? {
            self.work.report.report(ReportManagedWorkRequest {
                owner: name.into(),
                action_id: work.action_id,
                state: ManagedWorkState::Uncertain,
            })?;
        }

        Ok(())
    }

    fn failed(&mut self, name: &str, failure: &str) -> Result<()> {
        self.uncertain(name)?;
        let now = self.clock.now_millis();
        self.store.transaction(|records| {
            let mut plugin = records
                .registration(name)?
                .ok_or_else(|| Error::NotFound("plugin is absent".into()))?;
            plugin.running = false;
            plugin.failures = plugin.failures.saturating_add(1);
            plugin.retry_at = now.saturating_add(retry_delay(plugin.failures));
            plugin.last_failure = Some(failure.into());
            records.save(&plugin)
        })
    }

    fn cancel(
        &mut self,
        plugin: &PluginRegistration,
        session: &mut PluginSession,
        all: bool,
    ) -> Result<bool> {
        let work = self.unresolved(&plugin.name)?;
        let blocked = !work.is_empty();
        session
            .sent_cancellations
            .retain(|id| work.iter().any(|work| &work.action_id == id));
        if session.last_cancellation.elapsed() < Duration::from_millis(10) {
            return Ok(blocked);
        }
        if !work.is_empty() {
            let index = session.cancellation_cursor % work.len();
            session.cancellation_cursor = session.cancellation_cursor.wrapping_add(1);
            let mut work = work[index].clone();
            if all && !work.cancellation_requested {
                work = self.work.cancel.cancel(&plugin.name, &work.action_id)?;
            }
            if work.cancellation_requested
                && session.handshake
                && !session.sent_cancellations.contains(&work.action_id)
            {
                session.sent_cancellations.insert(work.action_id.clone());
                session
                    .process
                    .send(encode(&HostFrame::Cancel { work: work.into() })?)?;
            }
        }
        session.last_cancellation = Instant::now();

        Ok(blocked)
    }

    fn prune(&mut self) -> Result<()> {
        if self
            .work
            .list
            .list(ListManagedWorkRequest {
                owner: None,
                task: None,
            })?
            .iter()
            .any(|work| work.state.blocks_completion())
        {
            return Ok(());
        }
        let keep = self.options.retained_events.max(1);
        self.store.transaction(|records| {
            let mut cutoff = records.watermark()?.saturating_sub(keep).max(0);
            for plugin in records.registrations()? {
                cutoff = cutoff.min(plugin.cursor);
                for timer in records.timers(&plugin.name)? {
                    cutoff = cutoff.min(timer.origin_sequence);
                }
            }

            records.prune_events_before(cutoff)
        })
    }

    pub fn run(&mut self, shutdown: Arc<AtomicBool>) -> Result<()> {
        let registrations = self.store.transaction(|records| records.registrations())?;
        for plugin in registrations {
            self.uncertain(&plugin.name)?;
            self.store.transaction(|records| {
                let mut current = records
                    .registration(&plugin.name)?
                    .expect("registration exists");
                current.running = false;
                records.save(&current)
            })?;
        }
        let mut sessions: BTreeMap<String, PluginSession> = BTreeMap::new();
        let mut stopping = None;
        let mut last_prune = Instant::now();

        loop {
            if shutdown.load(Ordering::SeqCst) && stopping.is_none() {
                stopping = Some(Instant::now());
            }
            let registrations = self.store.transaction(|records| records.registrations())?;
            for mut plugin in registrations {
                let now = self.clock.now_millis();
                if stopping.is_none()
                    && plugin.enabled
                    && plugin.failures < 8
                    && now >= plugin.retry_at
                    && !sessions.contains_key(&plugin.name)
                {
                    let mut process = self.processes.spawn(&plugin)?;
                    process.send(encode(&HostFrame::Hello {
                        version: PROTOCOL_VERSION,
                        plugin: plugin.name.clone(),
                        max_frame_bytes: MAX_FRAME_BYTES,
                        state: plugin.state.clone().into(),
                    })?)?;
                    sessions.insert(
                        plugin.name.clone(),
                        PluginSession {
                            process,
                            handshake: false,
                            prefer_timer: true,
                            started_at: Instant::now(),
                            pending: None,
                            cancellation_cursor: 0,
                            sent_cancellations: std::collections::BTreeSet::new(),
                            draining_since: None,
                            last_cancellation: Instant::now() - Duration::from_secs(1),
                        },
                    );
                }
                let Some(mut session) = sessions.remove(&plugin.name) else {
                    continue;
                };
                let result = (|| {
                    for _ in 0..16 {
                        let Some(bytes) = session.process.receive()? else {
                            break;
                        };
                        handle_frame(
                            decode::<PluginFrame>(&bytes)?,
                            &plugin,
                            &mut session,
                            &mut self.store,
                            self.read.as_mut(),
                            &mut self.work,
                        )?;
                        plugin = self
                            .store
                            .transaction(|records| records.registration(&plugin.name))?
                            .ok_or_else(|| Error::NotFound("plugin is absent".into()))?;
                    }
                    let draining = !plugin.enabled || stopping.is_some();
                    if draining && session.draining_since.is_none() {
                        session.draining_since = Some(Instant::now());
                    }
                    let blocked = self.cancel(&plugin, &mut session, draining)?;
                    if draining && !blocked {
                        session.process.stop()?;
                        self.store.transaction(|records| {
                            let mut current = records
                                .registration(&plugin.name)?
                                .expect("registration exists");
                            current.running = false;
                            records.save(&current)
                        })?;

                        return Ok(false);
                    }
                    if session.draining_since.is_some_and(|started| {
                        started.elapsed() >= self.options.cancellation_timeout
                    }) && blocked
                    {
                        return Err(Error::ManagedWorkPending);
                    }
                    if !session.handshake
                        && session.started_at.elapsed() > self.options.response_timeout
                        || session.pending.as_ref().is_some_and(|pending| {
                            pending.sent_at.elapsed() > self.options.response_timeout
                        })
                    {
                        return Err(Error::Conflict("plugin response deadline exceeded".into()));
                    }
                    if !draining {
                        schedule(
                            &plugin,
                            &mut session,
                            &mut self.store,
                            self.events.as_mut(),
                            now,
                        )?;
                    }

                    Ok(true)
                })();
                match result {
                    Ok(true) => {
                        sessions.insert(plugin.name, session);
                    }
                    Ok(false) => {}
                    Err(error) => {
                        let _ = session.process.stop();
                        self.failed(
                            &plugin.name,
                            crate::failure_message::failure_message(&error),
                        )?;
                    }
                }
            }
            if let Some(started) = stopping {
                if sessions.is_empty() || started.elapsed() >= self.options.cancellation_timeout {
                    for (name, mut session) in sessions {
                        let _ = session.process.stop();
                        self.failed(&name, "managed cancellation unresolved during shutdown")?;
                    }
                    break;
                }
            }
            if last_prune.elapsed() >= Duration::from_secs(1) {
                self.prune()?;
                last_prune = Instant::now();
            }
            thread::sleep(self.options.poll_interval);
        }

        Ok(())
    }
}
