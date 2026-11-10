use crate::{pending_delivery::PendingDelivery, plugin_session::PluginSession};
use anyq_plugin_api::PluginRegistration;
use anyq_plugin_dispatch::Store;
use anyq_plugin_wire::{HostFrame, encode};
use anyq_queue_api::{ReadQueueEvents, Result};
use std::time::Instant;

pub(crate) fn schedule(
    plugin: &PluginRegistration,
    session: &mut PluginSession,
    store: &mut impl Store,
    events: &mut dyn ReadQueueEvents,
    now: i64,
) -> Result<()> {
    if !plugin.enabled || !session.handshake || session.pending.is_some() {
        return Ok(());
    }
    let timer = store.transaction(|records| {
        Ok(
            anyq_plugin_api::due_timers(&records.timers(&plugin.name)?, now, plugin.enabled)
                .into_iter()
                .next(),
        )
    })?;
    let next = events.after(plugin.cursor, 32)?;
    if let Some(timer) = timer
        .as_ref()
        .filter(|_| session.prefer_timer || next.is_empty())
    {
        send_timer(session, timer.clone())?;

        return Ok(());
    }
    for event in next {
        if !plugin
            .subscriptions
            .iter()
            .any(|kind| kind == event.snapshot.kind.as_str())
        {
            store.transaction(|records| {
                let mut current = records
                    .registration(&plugin.name)?
                    .expect("registration exists");
                current.cursor = event.sequence;
                records.save(&current)
            })?;

            continue;
        }
        let id = format!("event:{}", event.id());
        let sequence = event.sequence;
        session.process.send(encode(&HostFrame::Event {
            delivery_id: id.clone(),
            event: event.try_into()?,
        })?)?;
        session.prefer_timer = true;
        session.pending = Some(PendingDelivery {
            id,
            sequence: Some(sequence),
            timer: None,
            origin_sequence: sequence,
            sent_at: Instant::now(),
        });
        break;
    }

    if session.pending.is_none() {
        if let Some(timer) = timer {
            send_timer(session, timer)?;
        }
    }

    Ok(())
}

fn send_timer(session: &mut PluginSession, timer: anyq_plugin_api::PluginTimer) -> Result<()> {
    let id = format!("timer:{}:{}", timer.id, timer.deadline);
    session.process.send(encode(&HostFrame::Timer {
        delivery_id: id.clone(),
        timer: timer.clone().into(),
    })?)?;
    session.prefer_timer = false;
    session.pending = Some(PendingDelivery {
        id,
        sequence: None,
        timer: Some((timer.id, timer.deadline)),
        origin_sequence: timer.origin_sequence,
        sent_at: Instant::now(),
    });

    Ok(())
}
