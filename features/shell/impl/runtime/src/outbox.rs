use crate::stdio;
use anyq_plugin_wire::PluginFrame;
use anyq_queue_api::{Error, Result};
use std::{collections::VecDeque, io::Write};

pub(crate) struct Outbox<W> {
    writer: W,
    queued: VecDeque<PluginFrame>,
    pending: Option<String>,
}

impl<W: Write> Outbox<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            queued: VecDeque::new(),
            pending: None,
        }
    }

    pub fn has_capacity(&self) -> bool {
        // Reserve room for response-driven follow-ups and the host's single event.
        self.queued.len() < 2
    }

    pub fn acknowledge(&mut self, id: &str) {
        if self.pending.as_deref() == Some(id) {
            self.pending = None;
        }
    }

    pub fn send(&mut self, frame: PluginFrame) -> Result<()> {
        if let PluginFrame::Request { id, .. } = &frame {
            if self.pending.as_ref() == Some(id)
                || self.queued.iter().any(|frame| {
                    matches!(frame, PluginFrame::Request { id: queued, .. } if queued == id)
                })
            {
                return Ok(());
            }
        }
        if self.queued.len() >= 8 {
            return Err(Error::Invalid("shell output queue limit exceeded".into()));
        }

        self.queued.push_back(frame);
        self.flush()
    }

    pub fn flush(&mut self) -> Result<()> {
        while let Some(frame) = self.queued.front() {
            if matches!(frame, PluginFrame::Request { .. }) && self.pending.is_some() {
                break;
            }
            let frame = self.queued.pop_front().expect("front exists");
            if let PluginFrame::Request { id, .. } = &frame {
                self.pending = Some(id.clone());
            }

            stdio::send(&mut self.writer, frame)?;
        }

        Ok(())
    }
}
