use anyhow::{Result, bail};
use axum::http::HeaderMap;
use az_drive_core::model::EntryKey;

use crate::routes::{entry_key, header_or};

pub(crate) fn destination_key(headers: &HeaderMap) -> Result<EntryKey> {
    let destination = header_or(headers, "destination", "")?;
    let path = destination
        .split_once("://")
        .and_then(|(_, rest)| rest.split_once('/').map(|(_, path)| format!("/{path}")))
        .unwrap_or(destination);
    let path = path.trim_start_matches('/');
    let parts = path.split('/').collect::<Vec<_>>();
    if parts.len() < 3 || parts[0] != "dav" {
        bail!("invalid header: destination");
    }
    let relative = if parts.len() > 3 {
        parts[3..].join("/")
    } else {
        String::new()
    };
    entry_key(parts[1], parts[2], &relative)
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue};

    use super::destination_key;

    #[test]
    fn destination_key_parses_webdav_absolute_url() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "destination",
            HeaderValue::from_static("http://localhost:8788/dav/main/workspace/a.txt"),
        );

        let key = destination_key(&headers).expect("destination should parse");

        assert_eq!(key.remote_path(), "main/workspace/a.txt");
    }
}
