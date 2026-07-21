use reqwest::Url;
use sha2::{Digest, Sha256};

pub(crate) use az_str::transformation::trim_non_blank;

pub(crate) fn encode_url_component(value: &str) -> String {
    const UNRESERVED: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~";
    const HEX: &[u8; 16] = b"0123456789ABCDEF";

    let mut output = String::with_capacity(value.len().saturating_mul(3));
    for byte in value.as_bytes() {
        if UNRESERVED.contains(byte) {
            output.push(*byte as char);
        } else {
            output.push('%');
            output.push(HEX[(byte >> 4) as usize] as char);
            output.push(HEX[(byte & 0x0F) as usize] as char);
        }
    }
    output
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_string(&hasher.finalize())
}

pub(crate) fn hex_string(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0F) as usize] as char);
    }
    output
}

pub(crate) fn canonical_uri(url: &Url) -> String {
    let path = if url.path().is_empty() {
        "/"
    } else {
        url.path()
    };
    if path == "/" {
        return "/".to_owned();
    }
    path.split('/')
        .enumerate()
        .map(|(index, segment)| {
            if index == 0 {
                String::new()
            } else {
                encode_url_component(segment)
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

pub(crate) fn canonical_query_string(url: &Url) -> String {
    let mut pairs = url
        .query_pairs()
        .map(|(name, value)| {
            (
                encode_url_component(name.as_ref()),
                encode_url_component(value.as_ref()),
            )
        })
        .collect::<Vec<_>>();
    pairs.sort();
    pairs
        .into_iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("&")
}
