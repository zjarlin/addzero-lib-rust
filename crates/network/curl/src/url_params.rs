use reqwest::Url;
use std::collections::BTreeMap;

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
            is_uuid_like(segment) || is_numeric(segment) || is_dynamic_segment(segment)
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

fn is_uuid_like(segment: &str) -> bool {
    segment.len() >= 20
        && segment
            .chars()
            .all(|character| character.is_ascii_hexdigit() || character == '-')
}

fn is_numeric(segment: &str) -> bool {
    !segment.is_empty() && segment.chars().all(|character| character.is_ascii_digit())
}
