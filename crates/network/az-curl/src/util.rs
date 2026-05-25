use regex::Regex;
use std::sync::LazyLock;

static LINE_CONTINUATION_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\\\s*\r?\n").expect("line continuation regex should compile"));

pub(crate) fn normalize_command(command: &str) -> String {
    LINE_CONTINUATION_RE.replace_all(command, " ").into_owned()
}

pub(crate) fn normalize_header_name(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

pub(crate) fn looks_like_json(value: &str) -> bool {
    let trimmed = value.trim();
    (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
}
