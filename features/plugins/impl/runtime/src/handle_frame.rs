use crate::{capabilities::capability, plugin_session::PluginSession};
use anyq_plugin_api::{AcknowledgeRequest, PluginRegistration, TimerUpdate, validate_handshake};
use anyq_plugin_dispatch::{Store, acknowledge};
use anyq_plugin_wire::{HostFrame, PROTOCOL_VERSION, PluginFrame, TimerUpdateDto, encode};
use anyq_queue_api::{Error, GetTask, Result};

pub(crate) fn handle_frame(
    frame: PluginFrame,
    plugin: &PluginRegistration,
    session: &mut PluginSession,
    store: &mut impl Store,
    read: &mut dyn GetTask,
    work: &mut crate::WorkCapabilities,
) -> Result<()> {
    match frame {
        PluginFrame::Hello {
            version,
            subscriptions,
            capabilities,
        } => {
            if version != PROTOCOL_VERSION || session.handshake {
                return Err(Error::Invalid("incompatible plugin handshake".into()));
            }
            validate_handshake(&subscriptions, &capabilities)?;
            store.transaction(|records| {
                let mut current = records
                    .registration(&plugin.name)?
                    .ok_or_else(|| Error::NotFound("plugin is absent".into()))?;
                current.running = true;
                current.subscriptions = subscriptions;
                current.capabilities = capabilities;
                records.save(&current)
            })?;
            session.handshake = true;
        }
        _ if !session.handshake => {
            return Err(Error::Invalid("plugin handshake is required".into()));
        }
        PluginFrame::Ack {
            delivery_id,
            state,
            timers,
        } => {
            if !plugin.enabled {
                return Ok(());
            }

            let pending = session
                .pending
                .as_ref()
                .ok_or_else(|| Error::Invalid("unexpected plugin acknowledgement".into()))?;
            if delivery_id != pending.id
                || state.is_some() && !plugin.capabilities.iter().any(|cap| cap == "state")
                || !timers.is_empty() && !plugin.capabilities.iter().any(|cap| cap == "timers")
            {
                return Err(Error::Invalid(
                    "invalid plugin acknowledgement or undeclared capability".into(),
                ));
            }
            let timers = timers
                .into_iter()
                .map(|update| {
                    Ok(match update {
                        TimerUpdateDto::Put { timer } => {
                            TimerUpdate::Put(timer.into_timer(pending.origin_sequence)?)
                        }
                        TimerUpdateDto::Remove { id } => TimerUpdate::Remove(id),
                    })
                })
                .collect::<Result<_>>()?;
            acknowledge(
                store,
                AcknowledgeRequest {
                    plugin: plugin.name.clone(),
                    sequence: pending.sequence,
                    timer: pending.timer.clone(),
                    state: state.map(TryInto::try_into).transpose()?,
                    timers,
                },
            )?;
            session.pending = None;
        }
        PluginFrame::Request { id, request } => {
            if id.is_empty() || id.len() > 512 {
                return Err(Error::Invalid("invalid plugin request ID".into()));
            }
            let result = capability(plugin, request, read, work).unwrap_or_else(Into::into);
            session
                .process
                .send(encode(&HostFrame::Result { id, result })?)?;
        }
    }

    Ok(())
}
