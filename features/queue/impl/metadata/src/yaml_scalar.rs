use anyq_queue_api::{Error, MetadataNumber, MetadataValue, Result};
use yaml_rust2::{Yaml, scanner::TScalarStyle};

fn radix_number(input: &str) -> Result<Option<MetadataNumber>> {
    let unsigned = input.trim_start_matches(['+', '-']);
    let (radix, digits) = if let Some(digits) = unsigned.strip_prefix("0x") {
        (16_u32, digits)
    } else if let Some(digits) = unsigned.strip_prefix("0o") {
        (8, digits)
    } else {
        return Ok(None);
    };
    if digits.is_empty() || !digits.chars().all(|digit| digit.is_digit(radix)) {
        return Ok(None);
    }
    let digits = digits.trim_start_matches('0');
    if digits.len() > 342 {
        return Err(Error::Invalid(
            "metadata number is outside the finite range".into(),
        ));
    }
    let mut decimal = vec![0_u32];

    for digit in digits.chars() {
        let mut carry = digit.to_digit(radix).expect("validated radix digit");
        for position in decimal.iter_mut().rev() {
            let value = *position * radix + carry;
            *position = value % 10;
            carry = value / 10;
        }
        while carry != 0 {
            decimal.insert(0, carry % 10);
            carry /= 10;
        }
    }
    let mut text = if input.starts_with('-') {
        "-".into()
    } else {
        String::new()
    };
    text.extend(
        decimal
            .into_iter()
            .map(|digit| char::from_digit(digit, 10).expect("decimal digit")),
    );

    MetadataNumber::parse(&text).map(Some)
}

pub(crate) fn scalar(input: String, style: TScalarStyle) -> Result<MetadataValue> {
    if style != TScalarStyle::Plain {
        return Ok(MetadataValue::String(input));
    }
    if matches!(input.as_str(), "Null" | "NULL") {
        return Ok(MetadataValue::Null);
    }
    if let Some(number) = radix_number(&input)? {
        return Ok(MetadataValue::Number(number));
    }

    match Yaml::from_str(&input) {
        Yaml::Null => Ok(MetadataValue::Null),
        Yaml::Boolean(value) => Ok(MetadataValue::Bool(value)),
        Yaml::Integer(_) | Yaml::Real(_) => {
            let unsigned = input.strip_prefix('+').unwrap_or(&input);
            let (mantissa, exponent) = unsigned
                .split_once(['e', 'E'])
                .map_or((unsigned, None), |(mantissa, exponent)| {
                    (mantissa, Some(exponent))
                });
            let negative = mantissa.starts_with('-');
            let mantissa = mantissa.strip_prefix('-').unwrap_or(mantissa);
            let (integer, fraction) = mantissa
                .split_once('.')
                .map_or((mantissa, None), |(integer, fraction)| {
                    (integer, Some(fraction))
                });
            let integer = integer.trim_start_matches('0');
            let mut text = if negative { "-".into() } else { String::new() };
            text.push_str(if integer.is_empty() { "0" } else { integer });
            if let Some(fraction) = fraction {
                text.push('.');
                text.push_str(if fraction.is_empty() { "0" } else { fraction });
            }
            if let Some(exponent) = exponent {
                text.push('e');
                text.push_str(exponent);
            }

            MetadataNumber::parse(&text).map(MetadataValue::Number)
        }
        _ => Ok(MetadataValue::String(input)),
    }
}
