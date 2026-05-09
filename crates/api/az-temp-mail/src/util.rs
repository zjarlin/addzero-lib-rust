use sha2::{Digest, Sha256};

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

pub(crate) fn required_non_blank(value: &str, name: &str) -> crate::TempMailResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(crate::TempMailError::InvalidConfig(format!(
            "{name} cannot be blank"
        )));
    }
    Ok(trimmed.to_owned())
}

pub(crate) fn sha256_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex_string(&hasher.finalize())
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
