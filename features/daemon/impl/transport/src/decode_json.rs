use serde::de::DeserializeOwned;
use std::io;

// Metadata permits 64 logical levels. Typed wire envelopes add containers;
// bound physical JSON depth before deserializing those envelopes recursively.
const MAX_JSON_DEPTH: usize = 256;

pub(crate) fn decode_json<T: DeserializeOwned>(input: &[u8]) -> io::Result<T> {
    let mut depth = 0_usize;
    let mut quoted = false;
    let mut escaped = false;

    for byte in input {
        if quoted {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                quoted = false;
            }
            continue;
        }

        match byte {
            b'"' => quoted = true,
            b'{' | b'[' => {
                depth += 1;
                if depth > MAX_JSON_DEPTH {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "daemon JSON exceeds depth limit",
                    ));
                }
            }
            b'}' | b']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }

    let mut deserializer = serde_json::Deserializer::from_slice(input);
    deserializer.disable_recursion_limit();
    let value = T::deserialize(&mut deserializer)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    deserializer
        .end()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::decode_json;
    use anyq_daemon_api::MetadataDto;
    use std::collections::BTreeMap;

    #[test]
    fn maximum_logical_metadata_depth_fits_the_bounded_wire_decoder() {
        let mut value = anyq_queue_api::MetadataValue::Null;
        for _ in 1..anyq_queue_api::Metadata::MAX_DEPTH {
            value =
                anyq_queue_api::MetadataValue::Object(BTreeMap::from([("nested".into(), value)]));
        }
        let metadata = anyq_queue_api::Metadata::new(value).unwrap();
        let encoded = serde_json::to_vec(&MetadataDto::from(metadata.clone())).unwrap();

        let decoded: MetadataDto = decode_json(&encoded).unwrap();

        assert_eq!(
            anyq_queue_api::Metadata::try_from(decoded).unwrap(),
            metadata
        );
        let too_deep = format!("{}0{}", "[".repeat(257), "]".repeat(257));
        assert!(decode_json::<serde_json::Value>(too_deep.as_bytes()).is_err());
        assert!(decode_json::<String>(br#""[ { \" \\ ] }""#).is_ok());
    }
}
