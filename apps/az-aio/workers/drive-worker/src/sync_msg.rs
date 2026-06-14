//! Wire protocol messages and base64 helpers for the Drive CRDT sync worker.

use anyhow::Context;
use az_crdt::document::LineCrdtDocument;
use az_crdt::wire::LineCrdtVersion;
use base64::Engine;
use serde::{Deserialize, Serialize};

// ── Wire messages ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CrdtSyncMsg {
    Hello { device_id: String },
    HelloAck { peer_id: u64 },
    Open {
        remote_path: String,
        #[serde(default)]
        base_version: Option<String>,
    },
    Opened {
        remote_path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        snapshot: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        update: Option<String>,
        version: String,
    },
    Update {
        remote_path: String,
        update: String,
        base_version: Option<String>,
    },
    Close { remote_path: String },
    Error { message: String },
}

// ── Helpers ────────────────────────────────────────────────────────────

/// Returns a JSON error message as a `CrdtSyncMsg::Error`.
pub fn json_err(msg: &str) -> String {
    serde_json::to_string(&CrdtSyncMsg::Error {
        message: msg.to_owned(),
    })
    .unwrap_or_default()
}

/// Encode bytes as base64.
pub fn b64encode(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// Decode base64 string to bytes.
pub fn unbase64(encoded: &str) -> anyhow::Result<Vec<u8>> {
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .context("invalid base64")
}

// ── LineCrdtDocument extensions ────────────────────────────────────────

pub trait LineCrdtDocExt {
    fn version_bytes(&self) -> Vec<u8>;
    fn export_updates_since_bytes(&self, version: &[u8]) -> Vec<u8>;
}

impl LineCrdtDocExt for LineCrdtDocument {
    fn version_bytes(&self) -> Vec<u8> {
        self.version().into_bytes()
    }

    fn export_updates_since_bytes(&self, version: &[u8]) -> Vec<u8> {
        if version.is_empty() {
            return self
                .export_all_updates()
                .map(|u| u.into_bytes())
                .unwrap_or_default();
        }
        let vv = LineCrdtVersion::from_bytes(version.to_vec());
        self.export_updates_since(&vv)
            .map(|u| u.into_bytes())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_messages_serialize_with_type_tag() {
        let msg = CrdtSyncMsg::Hello {
            device_id: "test-device".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type""#));
        assert!(json.contains("hello"));
        assert!(json.contains("test-device"));
    }

    #[test]
    fn error_message_round_trips() {
        let err = CrdtSyncMsg::Error {
            message: "something broke".to_string(),
        };
        let json = serde_json::to_string(&err).unwrap();
        let parsed: CrdtSyncMsg = serde_json::from_str(&json).unwrap();
        match parsed {
            CrdtSyncMsg::Error { message } => assert_eq!(message, "something broke"),
            _ => panic!("expected Error variant"),
        }
    }
}
