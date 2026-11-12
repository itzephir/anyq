//! Versioned local protocol and passive typed queue handles.
pub const PROTOCOL_VERSION: u32 = 6;
pub const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_OBSERVATION_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

mod task_dto;
pub use task_dto::TaskDto;

mod topic_dto;
pub use topic_dto::TopicDto;

mod topic_queue_dto;
pub use topic_queue_dto::TopicQueueDto;

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

mod observed_task_dto;
pub use observed_task_dto::ObservedTaskDto;

mod observed_topic_dto;
pub use observed_topic_dto::ObservedTopicDto;

mod decode_task_state;

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
