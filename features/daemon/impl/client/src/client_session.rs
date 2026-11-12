use anyq_daemon_api::{
    MAX_OBSERVATION_RESPONSE_BYTES, PROTOCOL_VERSION, QueueRequest, QueueResponse, RequestFrame,
    ResponseFrame,
};
use anyq_daemon_transport::{FrameReader, Stream, send_frame};
use anyq_queue_api::{Error, Result};
use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

pub(crate) struct ClientSession {
    pub stream: Stream,
    pub cancelled: Arc<AtomicBool>,
    next_id: u64,
}

fn transport_error(error: io::Error, sent: bool) -> Error {
    if error.kind() == io::ErrorKind::Interrupted {
        Error::Interrupted
    } else if sent {
        Error::Storage(format!(
            "daemon response lost; operation outcome is uncertain; inspect queue state before retrying: {error}"
        ))
    } else {
        Error::Storage(format!("daemon request failed: {error}"))
    }
}

impl ClientSession {
    pub fn new(stream: Stream, cancelled: Arc<AtomicBool>) -> Self {
        Self {
            stream,
            cancelled,
            next_id: 0,
        }
    }

    pub fn request(&mut self, request: QueueRequest) -> Result<QueueResponse> {
        self.next_id += 1;
        let id = self.next_id;
        let observation = matches!(
            request,
            QueueRequest::OpenObservation { .. } | QueueRequest::ObserveQueue { .. }
        );
        let frame = RequestFrame {
            version: PROTOCOL_VERSION,
            id,
            request,
        };
        send_frame(
            &self.stream,
            &frame,
            &self.cancelled,
            Duration::from_secs(5),
        )
        .map_err(|error| transport_error(error, false))?;

        let mut reader = if observation {
            FrameReader::for_bounded_responses(MAX_OBSERVATION_RESPONSE_BYTES)
        } else {
            FrameReader::for_responses()
        };
        let received: io::Result<ResponseFrame> =
            reader.receive(&self.stream, &self.cancelled, Duration::from_secs(120));
        let response = match received {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                let cleanup = AtomicBool::new(false);
                let cancel = RequestFrame {
                    version: PROTOCOL_VERSION,
                    id,
                    request: QueueRequest::Cancel,
                };
                if send_frame(&self.stream, &cancel, &cleanup, Duration::from_secs(1)).is_ok() {
                    let _: io::Result<ResponseFrame> =
                        reader.receive(&self.stream, &cleanup, Duration::from_secs(5));
                }

                return Err(Error::Interrupted);
            }
            result => result.map_err(|error| transport_error(error, true))?,
        };

        if self.cancelled.load(Ordering::SeqCst) {
            return Err(Error::Interrupted);
        }

        if response.version != PROTOCOL_VERSION || response.id != id {
            return Err(Error::Storage("incompatible daemon response".into()));
        }

        match response.response {
            QueueResponse::Error(error) => Err(error.into()),
            response => Ok(response),
        }
    }
}
