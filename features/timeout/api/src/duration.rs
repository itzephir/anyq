use anyq_queue_api::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Duration(i64);

impl Duration {
    pub fn parse(value: &str) -> Result<Self> {
        let split = value.bytes().take_while(u8::is_ascii_digit).count();
        let (digits, unit) = value.split_at(split);
        let multiplier = match unit {
            "ms" => 1,
            "s" => 1_000,
            "m" => 60_000,
            "h" => 3_600_000,
            "d" => 86_400_000,
            _ => return Err(Error::Invalid("timeout requires ms, s, m, h or d".into())),
        };
        let milliseconds = digits
            .parse::<i64>()
            .ok()
            .filter(|value| *value > 0)
            .and_then(|value| value.checked_mul(multiplier))
            .ok_or_else(|| {
                Error::Invalid("timeout must be a positive bounded integer duration".into())
            })?;

        Ok(Self(milliseconds))
    }

    pub fn deadline(self, occurred_at: i64) -> Result<i64> {
        occurred_at
            .checked_add(self.0)
            .ok_or_else(|| Error::Invalid("timeout deadline overflow".into()))
    }
}
