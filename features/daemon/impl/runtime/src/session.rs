use crate::{QueueFactory, SessionSignals, route::route};
use anyq_daemon_api::{
    PROTOCOL_VERSION, QueueRequest, QueueResponse, QueueServices, RequestFrame, ResponseFrame,
};
use anyq_daemon_transport::{Endpoint, FrameReader, Stream, send_frame};
use anyq_queue_api::{Error, QueuePolicy, Result};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread,
    time::Duration,
};

pub(crate) struct Session {
    pub stream: Arc<Stream>,
    pub endpoint: Endpoint,
    pub factory: Arc<dyn QueueFactory>,
    pub signals: Arc<SessionSignals>,
    pub shutdown: Arc<AtomicBool>,
}

impl Session {
    fn observation_loop(&self, receiver: &Receiver<RequestFrame>, id: u64) -> Result<()> {
        let mut observer = match self
            .factory
            .observe(&self.endpoint.database, self.signals.cancelled.clone())
        {
            Ok(observer) => observer,
            Err(error) => return self.reply(id, QueueResponse::Error(error.into())),
        };
        self.reply(id, self.ready())?;

        loop {
            let frame = self.next(receiver)?;
            let response = match frame.request {
                QueueRequest::ObserveQueue {
                    max_entities,
                    max_text_bytes,
                } => {
                    match observer.observe(anyq_queue_api::ObserveQueueRequest {
                        max_entities,
                        max_text_bytes,
                    }) {
                        Ok(snapshot) => QueueResponse::Observation {
                            database_id: self.endpoint.identity.clone(),
                            snapshot: snapshot.into(),
                        },
                        Err(error) => QueueResponse::Error(error.into()),
                    }
                }
                _ => QueueResponse::Error(
                    Error::Invalid("observation sessions accept read-only snapshots only".into())
                        .into(),
                ),
            };
            self.reply(frame.id, response)?;
        }
    }

    fn reply(&self, id: u64, response: QueueResponse) -> Result<()> {
        send_frame(
            &self.stream,
            &ResponseFrame {
                version: PROTOCOL_VERSION,
                id,
                response,
            },
            &self.signals.closed,
            Duration::from_secs(5),
        )
        .map_err(|error| Error::Storage(error.to_string()))
    }

    fn ready(&self) -> QueueResponse {
        QueueResponse::Ready {
            pid: std::process::id(),
            database_id: self.endpoint.identity.clone(),
        }
    }

    fn next(&self, receiver: &Receiver<RequestFrame>) -> Result<RequestFrame> {
        loop {
            if self.signals.cancelled.load(Ordering::SeqCst) {
                return Err(Error::Interrupted);
            }

            match receiver.recv_timeout(Duration::from_millis(10)) {
                Ok(frame) if frame.version == PROTOCOL_VERSION => return Ok(frame),
                Ok(_) => {
                    return Err(Error::Invalid(
                        "incompatible daemon protocol version".into(),
                    ));
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => return Err(Error::Interrupted),
            }
        }
    }

    fn queue_loop(
        &self,
        receiver: &Receiver<RequestFrame>,
        id: u64,
        policy: QueuePolicy,
    ) -> Result<()> {
        let mut services: QueueServices = match self.factory.open(
            &self.endpoint.database,
            policy,
            self.signals.cancelled.clone(),
        ) {
            Ok(services) => services,
            Err(error) => {
                self.reply(id, QueueResponse::Error(error.into()))?;

                return Ok(());
            }
        };
        self.reply(id, self.ready())?;

        loop {
            let frame = self.next(receiver)?;
            let response = match route(frame.request, &mut services) {
                Ok(response) => response,
                Err(error) => QueueResponse::Error(error.into()),
            };
            self.reply(frame.id, response)?;
        }
    }

    fn handle(&self, receiver: &Receiver<RequestFrame>) -> Result<()> {
        let frame = self.next(receiver)?;
        match frame.request {
            QueueRequest::OpenObservation { database_id } => {
                if database_id != self.endpoint.identity {
                    return self.reply(
                        frame.id,
                        QueueResponse::Error(
                            Error::Invalid("daemon database identity mismatch".into()).into(),
                        ),
                    );
                }

                self.observation_loop(receiver, frame.id)
            }
            QueueRequest::Ping => self.reply(frame.id, self.ready()),
            QueueRequest::Stop => {
                self.reply(frame.id, QueueResponse::Stopped)?;
                self.shutdown.store(true, Ordering::SeqCst);

                Ok(())
            }
            QueueRequest::Open {
                database_id,
                policy,
                #[cfg(feature = "integration-test-hooks")]
                gate,
            } => {
                if database_id != self.endpoint.identity {
                    return self.reply(
                        frame.id,
                        QueueResponse::Error(
                            Error::Invalid("daemon database identity mismatch".into()).into(),
                        ),
                    );
                }

                #[cfg(feature = "integration-test-hooks")]
                {
                    let context = anyq_integration_test_hooks::GateContext::new(
                        gate.name,
                        gate.hit,
                        gate.directory,
                    );

                    anyq_integration_test_hooks::with_context(context, || {
                        self.queue_loop(receiver, frame.id, policy.into())
                    })
                }

                #[cfg(not(feature = "integration-test-hooks"))]
                self.queue_loop(receiver, frame.id, policy.into())
            }
            _ => self.reply(
                frame.id,
                QueueResponse::Error(
                    Error::Invalid("open a daemon session before queue operations".into()).into(),
                ),
            ),
        }
    }

    pub fn run(self) {
        let (sender, receiver) = mpsc::sync_channel(1);
        let stream = self.stream.clone();
        let signals = self.signals.clone();
        let reader = thread::spawn(move || {
            let mut reader = FrameReader::default();

            while let Ok(frame) =
                reader.receive::<RequestFrame>(&stream, &signals.closed, Duration::from_secs(30))
            {
                if matches!(frame.request, QueueRequest::Cancel) {
                    break;
                }

                if sender.try_send(frame).is_err() {
                    break;
                }
            }

            signals.cancelled.store(true, Ordering::SeqCst);
        });

        let _ = self.handle(&receiver);
        self.signals.close();
        let _ = reader.join();
    }
}
