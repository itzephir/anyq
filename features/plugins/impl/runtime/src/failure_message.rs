use anyq_queue_api::Error;

/// Status contains fixed diagnostics, never text interpolated from plugin input.
pub(crate) fn failure_message(error: &Error) -> &'static str {
    match error {
        Error::Invalid(message) if message == "incompatible plugin handshake" => {
            "incompatible plugin protocol version"
        }
        Error::Invalid(message) if message == "plugin diagnostic limit exceeded" => {
            "plugin diagnostic limit exceeded"
        }
        Error::Invalid(_) => "invalid plugin protocol, metadata or output limit",
        Error::ManagedWorkPending => "managed cancellation was not confirmed",
        Error::Conflict(_) => "plugin response deadline or acknowledgement conflict",
        Error::NotFound(_) => "plugin registration or action is unavailable",
        Error::Storage(_) => "plugin executable, protocol or storage IO failed",
        Error::Interrupted => "plugin operation was interrupted",
    }
}
