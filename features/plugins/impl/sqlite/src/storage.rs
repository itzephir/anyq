use anyq_queue_api::Error;

pub(crate) fn storage(error: rusqlite::Error) -> Error {
    Error::Storage(error.to_string())
}

pub(crate) fn conversion(error: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}
