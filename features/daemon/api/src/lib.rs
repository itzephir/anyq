//! Versioned local protocol and passive typed queue handles.
pub const PROTOCOL_VERSION: u32 = 7;
pub const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_OBSERVATION_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

pub use anyq_queue_wire::MetadataDto;

pub use anyq_queue_wire::CallerDirectoryDto;

pub use anyq_queue_wire::TaskDto;

pub use anyq_queue_wire::TopicDto;

pub use anyq_queue_wire::TopicQueueDto;

mod policy_dto;
pub use policy_dto::PolicyDto;

mod start_mode;
pub use start_mode::StartMode;

mod queue_error;
pub use queue_error::QueueError;

mod queue_request;
pub use queue_request::QueueRequest;

mod queue_snapshot_dto;
pub use queue_snapshot_dto::QueueSnapshotDto;

pub use anyq_queue_wire::ObservedTaskDto;

pub use anyq_queue_wire::ObservedTopicDto;

mod queue_response;
pub use queue_response::QueueResponse;

mod request_frame;
pub use request_frame::RequestFrame;

mod response_frame;
pub use response_frame::ResponseFrame;

pub use anyq_queue_api::QueueServices;

#[cfg(feature = "integration-test-hooks")]
mod gate_dto;
#[cfg(feature = "integration-test-hooks")]
pub use gate_dto::GateDto;

mod daemon_services;
pub use daemon_services::DaemonServices;
