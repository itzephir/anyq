use crate::{
    QueueFactory, SessionSignals, route::route, session_class::SessionClass,
    session_permit::SessionPermit,
};
use anyq_daemon_api::{
    DaemonServices, PROTOCOL_VERSION, QueueRequest, QueueResponse, RequestFrame, ResponseFrame,
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
    pub retained: Arc<AtomicBool>,
    pub permit: SessionPermit,
}

impl Session {
    fn require_capacity(&mut self, class: SessionClass) -> Result<()> {
        if self.permit.enter(class) {
            return Ok(());
        }

        Err(Error::Storage(match class {
            SessionClass::Waiting => "daemon waiting session limit reached".into(),
            _ => "daemon command session limit reached".into(),
        }))
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
        .map_err(|error| Error::Storage(error.to_string()))?;

        Ok(())
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
                Ok(frame) if frame.version == PROTOCOL_VERSION => {
                    self.signals.keep_alive.store(true, Ordering::SeqCst);

                    return Ok(frame);
                }
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
        &mut self,
        receiver: &Receiver<RequestFrame>,
        id: u64,
        policy: QueuePolicy,
        directory: anyq_queue_api::ExecutionDirectory,
        retained_services: &mut Option<DaemonServices>,
    ) -> Result<()> {
        let services = match self.factory.open(
            &self.endpoint.database,
            policy,
            self.signals.cancelled.clone(),
            directory,
        ) {
            Ok(services) => services,
            Err(error) => {
                self.reply(id, QueueResponse::Error(error.into()))?;

                return Ok(());
            }
        };

        *retained_services = Some(services);
        self.reply(id, self.ready())?;
        let services = retained_services.as_mut().expect("queue services opened");

        loop {
            let frame = self.next(receiver)?;
            let class = match &frame.request {
                QueueRequest::CheckTaskRunning { .. } | QueueRequest::StartTask { .. } => {
                    self.permit.class
                }
                _ => SessionClass::Command,
            };
            if let Err(error) = self.require_capacity(class) {
                self.reply(frame.id, QueueResponse::Error(error.into()))?;

                return Ok(());
            }

            let response = match route(frame.request, services) {
                Ok(response) => response,
                Err(error) => QueueResponse::Error(error.into()),
            };
            let waiting = matches!(&response, QueueResponse::Running(false))
                || matches!(&response, QueueResponse::Started(outcome) if outcome == "busy");
            if waiting {
                if let Err(error) = self.require_capacity(SessionClass::Waiting) {
                    self.reply(frame.id, QueueResponse::Error(error.into()))?;

                    return Ok(());
                }
            }

            self.reply(frame.id, response)?;
        }
    }

    fn handle(
        &mut self,
        receiver: &Receiver<RequestFrame>,
        services: &mut Option<DaemonServices>,
    ) -> Result<()> {
        let frame = self.next(receiver)?;
        match frame.request {
            QueueRequest::Ping => self.reply(frame.id, self.ready()),
            QueueRequest::Stop => {
                self.reply(frame.id, QueueResponse::Stopped)?;
                self.shutdown.store(true, Ordering::SeqCst);

                Ok(())
            }
            QueueRequest::Open {
                database_id,
                policy,
                caller_directory,
                #[cfg(feature = "integration-test-hooks")]
                gate,
            } => {
                let caller_directory: std::path::PathBuf = caller_directory.try_into()?;
                if !caller_directory.is_absolute() {
                    return self.reply(
                        frame.id,
                        QueueResponse::Error(
                            Error::Invalid("caller directory must be absolute".into()).into(),
                        ),
                    );
                }

                if database_id != self.endpoint.identity {
                    return self.reply(
                        frame.id,
                        QueueResponse::Error(
                            Error::Invalid("daemon database identity mismatch".into()).into(),
                        ),
                    );
                }

                if let Err(error) = self.require_capacity(SessionClass::Command) {
                    return self.reply(frame.id, QueueResponse::Error(error.into()));
                }

                #[cfg(feature = "integration-test-hooks")]
                {
                    let context = anyq_integration_test_hooks::GateContext::new(
                        gate.name,
                        gate.hit,
                        gate.directory,
                    );

                    anyq_integration_test_hooks::with_context(context, || {
                        self.queue_loop(
                            receiver,
                            frame.id,
                            policy.into(),
                            anyq_daemon_api::CallerDirectoryDto::from(caller_directory).into(),
                            services,
                        )
                    })
                }

                #[cfg(not(feature = "integration-test-hooks"))]
                self.queue_loop(
                    receiver,
                    frame.id,
                    policy.into(),
                    anyq_daemon_api::CallerDirectoryDto::from(caller_directory).into(),
                    services,
                )
            }
            _ => self.reply(
                frame.id,
                QueueResponse::Error(
                    Error::Invalid("open a daemon session before queue operations".into()).into(),
                ),
            ),
        }
    }

    pub fn run(mut self) {
        let (sender, receiver) = mpsc::sync_channel(1);
        let stream = self.stream.clone();
        let signals = self.signals.clone();
        let reader = thread::spawn(move || {
            let mut reader = FrameReader::default();

            while let Ok(frame) = reader.receive_request::<RequestFrame>(
                &stream,
                &signals.closed,
                &signals.keep_alive,
                Duration::from_secs(30),
            ) {
                if matches!(frame.request, QueueRequest::Cancel) {
                    break;
                }

                // Replies can be followed by a long CLI output pause. Keep a
                // complete session alive until EOF, cancellation or shutdown.
                signals.keep_alive.store(true, Ordering::SeqCst);
                if sender.try_send(frame).is_err() {
                    break;
                }
            }

            signals.cancelled.store(true, Ordering::SeqCst);
        });

        let mut services = None;
        let _ = self.handle(&receiver, &mut services);
        self.signals.close();
        let _ = reader.join();

        let retain = services.is_some() && !self.retained.swap(true, Ordering::SeqCst);
        let shutdown = self.shutdown.clone();
        drop(self);

        // Keep one initialized storage context for the owner's lifetime. Client
        // EOF releases its stream and permit before this otherwise idle wait.
        if retain {
            while !shutdown.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(10));
            }
        }

        drop(services);
    }
}
