pub(crate) fn storage(error: rusqlite::Error) -> anyq_queue_api::Error {
    anyq_queue_api::Error::Storage(error.to_string())
}
