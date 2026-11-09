use crate::parse_name;

pub fn format_name(value: &str) -> String {
    if parse_name(value).is_ok_and(|parsed| parsed == value) {
        value.into()
    } else {
        format!("`{value}`")
    }
}
