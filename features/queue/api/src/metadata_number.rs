use crate::{Error, Metadata, Result};

/// An exact finite decimal value with a normalized, JSON-compatible spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataNumber(String);

impl MetadataNumber {
    pub fn parse(input: &str) -> Result<Self> {
        let invalid =
            || Error::Invalid("metadata number must be finite JSON decimal syntax".into());
        if input.len() > Metadata::MAX_BYTES || !input.is_ascii() {
            return Err(invalid());
        }

        let bytes = input.as_bytes();
        let mut index = usize::from(bytes.first() == Some(&b'-'));
        match bytes.get(index) {
            Some(b'0') => index += 1,
            Some(b'1'..=b'9') => {
                index += 1;
                while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                    index += 1;
                }
            }
            _ => return Err(invalid()),
        }
        let integer_end = index;
        let mut fraction_length = 0;
        if bytes.get(index) == Some(&b'.') {
            index += 1;
            let start = index;
            while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                index += 1;
            }
            fraction_length = index - start;
            if fraction_length == 0 {
                return Err(invalid());
            }
        }
        let mantissa_end = index;
        let mut exponent = 0_i32;
        if matches!(bytes.get(index), Some(b'e' | b'E')) {
            index += 1;
            let start = index;
            if matches!(bytes.get(index), Some(b'+' | b'-')) {
                index += 1;
            }
            let digits = index;
            while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                index += 1;
            }
            if digits == index {
                return Err(invalid());
            }
            exponent = input[start..index].parse().map_err(|_| invalid())?;
        }
        if index != input.len() || !input.parse::<f64>().is_ok_and(f64::is_finite) {
            return Err(invalid());
        }

        let negative = input.starts_with('-');
        let start = usize::from(negative);
        let mut digits = input[start..integer_end].to_owned();
        if fraction_length != 0 {
            digits.push_str(&input[integer_end + 1..mantissa_end]);
        }
        let digits = digits.trim_start_matches('0');
        if digits.is_empty() {
            return Ok(Self("0".into()));
        }

        let significant = digits.trim_end_matches('0');
        let scale = i64::from(exponent) - fraction_length as i64
            + (digits.len() - significant.len()) as i64;
        let point = significant.len() as i64 + scale;
        let mut normalized = if negative { "-".into() } else { String::new() };
        if scale >= 0 && point <= 21 {
            normalized.push_str(significant);
            normalized.extend(std::iter::repeat_n('0', scale as usize));
        } else if point > 0 && point < significant.len() as i64 {
            normalized.push_str(&significant[..point as usize]);
            normalized.push('.');
            normalized.push_str(&significant[point as usize..]);
        } else if (-5..=0).contains(&point) {
            normalized.push_str("0.");
            normalized.extend(std::iter::repeat_n('0', (-point) as usize));
            normalized.push_str(significant);
        } else {
            normalized.push_str(&significant[..1]);
            if significant.len() > 1 {
                normalized.push('.');
                normalized.push_str(&significant[1..]);
            }
            normalized.push('e');
            normalized.push_str(&(point - 1).to_string());
        }

        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
