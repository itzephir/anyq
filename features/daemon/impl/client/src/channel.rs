use crate::{client_session::ClientSession, start_owner::start_owner};
use anyq_daemon_api::{QueueRequest, QueueResponse};
use anyq_daemon_transport::Endpoint;
use anyq_queue_api::{Error, QueuePolicy, Result};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, atomic::AtomicBool},
};

#[derive(Clone)]
pub struct Channel {
    session: Rc<RefCell<ClientSession>>,
}

impl Channel {
    pub fn open_observer(endpoint: &Endpoint, cancelled: Arc<AtomicBool>) -> Result<Self> {
        if !endpoint.database.is_file() {
            return Err(Error::NotFound(
                "selected queue database does not exist".into(),
            ));
        }
        let (endpoint, stream) =
            start_owner(endpoint, &cancelled).map_err(|error| Error::Storage(error.to_string()))?;
        let channel = Self {
            session: Rc::new(RefCell::new(ClientSession::new(stream, cancelled))),
        };

        match channel.request(QueueRequest::OpenObservation {
            database_id: endpoint.identity.clone(),
        })? {
            QueueResponse::Ready { database_id, .. } if database_id == endpoint.identity => {
                Ok(channel)
            }
            _ => Err(Self::unexpected()),
        }
    }

    pub fn open(
        endpoint: &Endpoint,
        policy: QueuePolicy,
        cancelled: Arc<AtomicBool>,
    ) -> Result<Self> {
        let caller_directory =
            std::env::current_dir().map_err(|error| Error::Storage(error.to_string()))?;
        let (endpoint, stream) = start_owner(endpoint, &cancelled).map_err(|error| {
            if error.kind() == std::io::ErrorKind::Interrupted {
                Error::Interrupted
            } else {
                Error::Storage(error.to_string())
            }
        })?;
        let channel = Self {
            session: Rc::new(RefCell::new(ClientSession::new(stream, cancelled))),
        };
        let response = channel.request(QueueRequest::Open {
            database_id: endpoint.identity.clone(),
            policy: policy.into(),
            caller_directory: caller_directory.into(),
            #[cfg(feature = "integration-test-hooks")]
            gate: anyq_daemon_api::GateDto {
                name: std::env::var("ANYQ_TEST_CHECKPOINT").ok(),
                hit: std::env::var("ANYQ_TEST_CHECKPOINT_HIT")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(1),
                directory: std::env::var_os("ANYQ_TEST_CHECKPOINT_DIR").map(Into::into),
            },
        })?;

        match response {
            QueueResponse::Ready { database_id, .. } if database_id == endpoint.identity => {
                Ok(channel)
            }
            _ => Err(Self::unexpected()),
        }
    }

    pub fn request(&self, request: QueueRequest) -> Result<QueueResponse> {
        self.session.borrow_mut().request(request)
    }

    pub fn unexpected() -> Error {
        Error::Storage("unexpected daemon response".into())
    }
}
