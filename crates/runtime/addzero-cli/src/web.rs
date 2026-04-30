pub mod robots;

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use encoding_rs::{Encoding, UTF_8};
use regex::Regex;
use reqwest::Url;
use reqwest::blocking::{Client, Response};
use reqwest::header::CONTENT_TYPE;
use scraper::{ElementRef, Html, Selector};

pub fn build_client(user_agent: &str) -> Result<Client> {
    Client::builder()
        .user_agent(user_agent.to_owned())
        .timeout(Duration::from_secs(20))
        .build()
        .context("build HTTP client")
}

pub fn parse_selector(value: &str, kind: &str) -> Result<Selector> {
    Selector::parse(value).map_err(|error| anyhow::anyhow!("invalid {kind} `{value}`: {error}"))
}

pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .with_context(|| format!("create output directory: {}", parent.display()))
}

pub fn decode_html(response: Response) -> Result<String> {
    let header_charset = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(parse_charset_from_content_type)
        .map(str::to_owned);

    let bytes = response.bytes().context("read response body")?;
    let meta_charset = detect_meta_charset(bytes.as_ref());
    let encoding = header_charset
        .as_deref()
        .or(meta_charset.as_deref())
        .and_then(|label| Encoding::for_label(label.as_bytes()))
        .unwrap_or(UTF_8);

    let (decoded, _, _) = encoding.decode(bytes.as_ref());
    Ok(decoded.into_owned())
}

pub fn first_non_empty_text(document: &Html, selector: &Selector) -> Option<String> {
    document
        .select(selector)
        .find_map(|element| normalized_text(&element))
}

pub fn normalized_text(element: &ElementRef<'_>) -> Option<String> {
    let text = element
        .text()
        .map(normalize_inline_whitespace)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let text = text.trim().to_owned();
    if text.is_empty() { None } else { Some(text) }
}

pub fn normalize_inline_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn href_from_element(base_url: &Url, element: &ElementRef<'_>) -> Option<Url> {
    let href = element.value().attr("href")?;
    base_url.join(href).ok()
}

pub fn same_origin(left: &Url, right: &Url) -> bool {
    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

pub fn manifest_path(output: &Path, default_stem: &str) -> PathBuf {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let stem = output_stem(output, default_stem);
    parent.join(format!("{stem}.manifest.json"))
}

pub fn output_stem(output: &Path, default_stem: &str) -> String {
    output
        .file_stem()
        .and_then(OsStr::to_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(default_stem)
        .to_owned()
}

fn parse_charset_from_content_type(content_type: &str) -> Option<&str> {
    content_type
        .split(';')
        .map(str::trim)
        .find_map(|part| part.strip_prefix("charset="))
}

fn detect_meta_charset(bytes: &[u8]) -> Option<String> {
    let probe = String::from_utf8_lossy(&bytes[..bytes.len().min(4096)]);
    let utf8_meta = Regex::new(r#"(?i)<meta[^>]+charset=["']?([a-z0-9_\-]+)"#).ok()?;
    if let Some(captures) = utf8_meta.captures(&probe) {
        return captures.get(1).map(|value| value.as_str().to_owned());
    }

    let content_type_meta =
        Regex::new(r#"(?i)<meta[^>]+content=["'][^"']*charset=([a-z0-9_\-]+)[^"']*["']"#).ok()?;
    content_type_meta
        .captures(&probe)
        .and_then(|captures| captures.get(1))
        .map(|value| value.as_str().to_owned())
}
