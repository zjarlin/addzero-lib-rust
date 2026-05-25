use regex::Regex;
use reqwest::Url;
use std::collections::BTreeMap;
use std::sync::LazyLock;

static UUID_LIKE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^[a-f0-9\-]{20,}$").expect("uuid regex should compile"));
static NUMERIC_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\d+$").expect("numeric regex should compile"));

pub(crate) fn extract_query_params(url: &Url) -> BTreeMap<String, String> {
    url.query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect()
}

pub(crate) fn extract_path_params(url: &Url) -> Vec<String> {
    url.path_segments()
        .into_iter()
        .flatten()
        .filter(|segment| !segment.is_empty())
        .filter(|segment| !is_version_segment(segment))
        .filter(|segment| {
            UUID_LIKE_RE.is_match(segment)
                || NUMERIC_RE.is_match(segment)
                || is_dynamic_segment(segment)
        })
        .map(ToOwned::to_owned)
        .collect()
}

fn is_version_segment(segment: &str) -> bool {
    let Some(rest) = segment.strip_prefix('v') else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|ch| ch.is_ascii_digit())
}

fn is_dynamic_segment(segment: &str) -> bool {
    let is_token = segment
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-');
    let has_letter = segment.chars().any(|ch| ch.is_ascii_alphabetic());
    let has_digit = segment.chars().any(|ch| ch.is_ascii_digit());
    is_token && has_letter && has_digit
}
