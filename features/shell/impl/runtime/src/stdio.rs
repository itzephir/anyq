use anyq_plugin_wire::{HostFrame, MAX_FRAME_BYTES, PluginFrame, decode, encode};
use anyq_queue_api::{Error, Result};
use std::io::{BufRead, Write};

pub(crate) fn receive(input: &mut impl BufRead) -> Result<Option<HostFrame>> {
    let mut bytes = Vec::new();
    loop {
        let available = input
            .fill_buf()
            .map_err(|error| Error::Storage(error.to_string()))?;
        if available.is_empty() {
            return if bytes.is_empty() {
                Ok(None)
            } else {
                Err(Error::Invalid("truncated plugin input".into()))
            };
        }
        let count = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |index| index + 1);
        if bytes.len().saturating_add(count) > MAX_FRAME_BYTES + 1 {
            return Err(Error::Invalid("plugin input frame limit exceeded".into()));
        }
        bytes.extend_from_slice(&available[..count]);
        input.consume(count);
        if bytes.last() == Some(&b'\n') {
            bytes.pop();
            return decode(&bytes).map(Some);
        }
    }
}

pub(crate) fn send(output: &mut impl Write, frame: PluginFrame) -> Result<()> {
    output
        .write_all(&encode(&frame)?)
        .and_then(|_| output.write_all(b"\n"))
        .and_then(|_| output.flush())
        .map_err(|error| Error::Storage(error.to_string()))
}
