//! Subscription parser dispatch for Clash YAML, direct URI lists, and base64 URI lists.

use crate::clash::parse_clash_yaml;
use crate::types::{ProxyError, ProxyNode, ProxyResult};
use base64::Engine;

automod::dir!(pub "src/parser");

/// Parses a single supported proxy URI into a proxy node.
///
/// Supported URI schemes include `ss`, `vmess`, `vless`, `trojan`,
/// `hysteria2`, `hy2`, `tuic`, and `wireguard`.
///
/// # Errors
///
/// Returns an error when the scheme is unsupported or the URI does not contain
/// enough data to build a Clash-compatible proxy entry.
pub fn parse_proxy_uri(input: &str) -> ProxyResult<ProxyNode> {
    uri::parse_proxy_uri(input)
}

/// Parses newline-separated proxy URI subscriptions.
///
/// Empty lines and lines without a supported URI scheme are ignored.
///
/// # Errors
///
/// Returns an error when a supported URI line is malformed, or when no usable
/// proxy nodes are found.
pub fn parse_uri_lines(input: &str) -> ProxyResult<Vec<ProxyNode>> {
    uri::parse_uri_lines(input)
}

/// Parses subscription text into supported proxy nodes.
///
/// `content_type` is optional and is used only as a hint. The response body is
/// still inspected so subscriptions with incorrect headers can be parsed.
///
/// # Errors
///
/// Returns an error when the body is empty, has invalid YAML for an apparent
/// YAML subscription, has invalid URI data, or contains no usable proxy nodes.
pub fn parse_subscription(body: &str, content_type: Option<&str>) -> ProxyResult<Vec<ProxyNode>> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(ProxyError::NoUsableNodes);
    }

    if looks_like_clash_yaml(trimmed, content_type) {
        return parse_clash_yaml(trimmed);
    }

    if contains_supported_uri(trimmed) {
        return parse_uri_lines(trimmed);
    }

    if let Some(decoded) = decode_base64_text(trimmed) {
        let decoded = decoded.trim();
        if looks_like_clash_yaml(decoded, None) {
            return parse_clash_yaml(decoded);
        }
        return parse_uri_lines(decoded);
    }

    let yaml_result = parse_clash_yaml(trimmed);
    if yaml_result.as_ref().is_ok_and(|nodes| !nodes.is_empty()) {
        return yaml_result;
    }

    Err(ProxyError::NoUsableNodes)
}

fn looks_like_clash_yaml(body: &str, content_type: Option<&str>) -> bool {
    let content_type_is_yaml = content_type.is_some_and(|value| {
        let value = value.to_ascii_lowercase();
        value.contains("yaml") || value.contains("yml")
    });

    content_type_is_yaml
        || body.starts_with("proxies:")
        || body.contains("\nproxies:")
        || body.contains("\r\nproxies:")
}

fn contains_supported_uri(body: &str) -> bool {
    body.lines().any(|line| {
        let line = line.trim_start();
        matches!(
            line.split_once("://").map(|(scheme, _)| scheme),
            Some("ss" | "vmess" | "vless" | "trojan" | "hysteria2" | "hy2" | "tuic" | "wireguard")
        )
    })
}

pub(crate) fn decode_base64_text(input: &str) -> Option<String> {
    let compact: String = input.chars().filter(|ch| !ch.is_whitespace()).collect();
    if compact.is_empty() {
        return None;
    }

    [
        base64::engine::general_purpose::STANDARD,
        base64::engine::general_purpose::STANDARD_NO_PAD,
        base64::engine::general_purpose::URL_SAFE,
        base64::engine::general_purpose::URL_SAFE_NO_PAD,
    ]
    .iter()
    .find_map(|engine| {
        engine
            .decode(compact.as_bytes())
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
    })
}
