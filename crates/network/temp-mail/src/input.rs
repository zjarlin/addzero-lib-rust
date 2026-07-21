use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use az_str::sanitize::ascii_alphanumeric_or;
pub(crate) use az_str::transformation::trim_non_blank;

pub(crate) fn required_non_blank(value: &str, name: &str) -> anyhow::Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        anyhow::bail!("invalid config: {name} cannot be blank");
    }
    Ok(trimmed.to_owned())
}

pub(crate) fn sha256_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex_string(&hasher.finalize())
}

pub(crate) fn sanitize_local_part(prefix: &str) -> String {
    ascii_alphanumeric_or(prefix, "az")
}

pub(crate) fn random_alpha_numeric(length: usize) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    const ALPHABET_LEN: u64 = 36;
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let mut state = seed_random_state(COUNTER.fetch_add(1, Ordering::Relaxed));
    let mut output = String::with_capacity(length);

    while output.len() < length {
        state = xorshift64(state);
        let reduced = state % ALPHABET_LEN;
        let Ok(index) = usize::try_from(reduced) else {
            continue;
        };
        output.push(ALPHABET[index] as char);
    }

    output
}

fn hex_string(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0F) as usize] as char);
    }
    output
}

fn seed_random_state(counter: u64) -> u64 {
    let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() ^ u64::from(duration.subsec_nanos()).rotate_left(32),
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
