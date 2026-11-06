use anyq_queue_dispatch::ports::Tokens;

pub(crate) struct UuidTokens;

impl Tokens for UuidTokens {
    fn next(&mut self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
