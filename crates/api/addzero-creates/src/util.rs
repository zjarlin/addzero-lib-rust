use reqwest::Url;
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn trim_non_blank(value: Option<&str>) -> Option<&str> {
    value.and_then(|item| {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

pub(crate) fn non_blank(value: Option<&str>) -> Option<&str> {
    trim_non_blank(value)
}

pub(crate) fn encode_url_component(value: &str) -> String {
    const UNRESERVED: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~";
    let mut output = String::new();
    for byte in value.as_bytes() {
        if UNRESERVED.contains(byte) {
            output.push(*byte as char);
        } else {
            output.push_str(&format!("%{:02X}", byte));
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
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
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
    pairs.sort_by(|left, right| left.cmp(right));
    pairs
        .into_iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("&")
}

pub(crate) fn default_user_agent() -> String {
    format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}

pub(crate) fn sanitize_prefix(prefix: &str) -> String {
    let sanitized = prefix
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>();

    if sanitized.is_empty() {
        "az".to_owned()
    } else {
        sanitized
    }
}

pub(crate) fn random_alpha_numeric(length: usize) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let mut state = seed_random_state(COUNTER.fetch_add(1, Ordering::Relaxed));
    let mut output = String::with_capacity(length);

    while output.len() < length {
        state = xorshift64(state);
        let index = (state as usize) % ALPHABET.len();
        output.push(ALPHABET[index] as char);
    }

    output
}

fn seed_random_state(counter: u64) -> u64 {
    let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos() as u64,
        Err(_) => 0,
    };
    let mixed = now ^ counter.rotate_left(19) ^ 0x9E37_79B9_7F4A_7C15;
    if mixed == 0 {
        0xA5A5_A5A5_A5A5_A5A5
    } else {
        mixed
    }
}

fn xorshift64(mut state: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}
