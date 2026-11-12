//! Versioned local protocol and passive typed queue handles.
pub const PROTOCOL_VERSION: u32 = 2;
pub const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;

mod metadata_dto;
pub use metadata_dto::MetadataDto;

mod caller_directory_dto;
pub use caller_directory_dto::CallerDirectoryDto;

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
