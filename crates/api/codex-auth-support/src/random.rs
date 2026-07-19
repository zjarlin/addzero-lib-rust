use anyhow::{Context, anyhow};
use ring::rand::{SecureRandom, SystemRandom};

const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%&*";
const LOCAL_PART: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

pub(crate) fn random_local_part(length: usize) -> anyhow::Result<String> {
    random_string(length.max(3), LOCAL_PART)
}

pub(crate) fn random_password(length: usize) -> anyhow::Result<String> {
    let length = length.max(8);
    let mut chars = Vec::with_capacity(length);
    chars.push(random_char(LOWER)?);
    chars.push(random_char(UPPER)?);
    chars.push(random_char(DIGITS)?);
    chars.push(random_char(SYMBOLS)?);

    let mut all = Vec::new();
    all.extend_from_slice(LOWER);
    all.extend_from_slice(UPPER);
    all.extend_from_slice(DIGITS);
    all.extend_from_slice(SYMBOLS);

    while chars.len() < length {
        chars.push(random_char(&all)?);
    }

    shuffle(&mut chars)?;
    String::from_utf8(chars)
        .map_err(|_| anyhow!("invalid response: generated password was not UTF-8"))
}

pub(crate) fn random_bytes<const N: usize>() -> anyhow::Result<[u8; N]> {
    let rng = SystemRandom::new();
    let mut bytes = [0u8; N];
    rng.fill(&mut bytes)
        .map_err(|_| anyhow!("crypto random generation failed"))
        .context("failed to generate auth support random bytes")?;
    Ok(bytes)
}

fn random_string(length: usize, alphabet: &[u8]) -> anyhow::Result<String> {
    let mut chars = Vec::with_capacity(length);
    for _ in 0..length {
        chars.push(random_char(alphabet)?);
    }
    String::from_utf8(chars)
        .map_err(|_| anyhow!("invalid response: generated string was not UTF-8"))
}

fn random_char(alphabet: &[u8]) -> anyhow::Result<u8> {
    let bytes = random_bytes::<1>()?;
    let index = usize::from(bytes[0]) % alphabet.len();
    Ok(alphabet[index])
}

fn shuffle(values: &mut [u8]) -> anyhow::Result<()> {
    if values.len() < 2 {
        return Ok(());
    }

    for i in (1..values.len()).rev() {
        let bytes = random_bytes::<1>()?;
        let j = usize::from(bytes[0]) % (i + 1);
        values.swap(i, j);
    }
    Ok(())
}
