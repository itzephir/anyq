//! Explicit local plugin boundary.

mod managed_work_dto;
pub use managed_work_dto::{ManagedWorkDto, work_state};

mod timer_dto;
pub use timer_dto::TimerDto;

mod timer_update_dto;
pub use timer_update_dto::TimerUpdateDto;

mod capability_request;
pub use capability_request::CapabilityRequest;

mod capability_result;
pub use capability_result::CapabilityResult;

mod host_frame;
pub use host_frame::HostFrame;

mod plugin_frame;
pub use plugin_frame::PluginFrame;

mod plugin_status_dto;
pub use plugin_status_dto::PluginStatusDto;

mod codec;
pub use codec::{MAX_FRAME_BYTES, PROTOCOL_VERSION, decode, encode};
