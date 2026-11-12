use anyq_queue_api::{Error, Result};
use serde::{Serialize, de::DeserializeOwned};

pub const PROTOCOL_VERSION: u32 = 2;
pub const MAX_FRAME_BYTES: usize = 16 * 1024 * 1024;

pub fn encode<T: Serialize>(frame: &T) -> Result<Vec<u8>> {
    let value = serde_json::to_vec(frame)
        .map_err(|_| Error::Invalid("cannot encode plugin frame".into()))?;
    if value.len() > MAX_FRAME_BYTES {
        return Err(Error::Invalid("plugin frame limit exceeded".into()));
    }

    Ok(value)
}

pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    if bytes.len() > MAX_FRAME_BYTES {
        return Err(Error::Invalid("plugin frame limit exceeded".into()));
    }
    let (mut depth, mut string, mut escaped) = (0_u16, false, false);
    for byte in bytes {
        if string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                string = false;
            }
        } else {
            match byte {
                b'"' => string = true,
                b'{' | b'[' => {
                    depth += 1;
                    if depth > 256 {
                        return Err(Error::Invalid("plugin frame depth exceeded".into()));
                    }
                }
                b'}' | b']' => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
    }
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    decoder.disable_recursion_limit();
    let result =
        T::deserialize(&mut decoder).map_err(|_| Error::Invalid("invalid plugin frame".into()))?;
    decoder
        .end()
        .map_err(|_| Error::Invalid("invalid plugin frame".into()))?;

    Ok(result)
}
