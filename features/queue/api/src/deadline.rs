use crate::{Error, Result};

pub fn deadline(now: i64, duration_ms: i64) -> Result<i64> {
    if !(1..=86_400_000).contains(&duration_ms) {
        return Err(Error::Invalid(
            "duration must be in 1..86400000 milliseconds".into(),
        ));
    }

    now.checked_add(duration_ms)
        .ok_or_else(|| Error::Invalid("timestamp overflow".into()))
}
