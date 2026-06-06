use serde::{Deserialize, Serialize};

use crate::{
    error::SyncResult,
    sync_index::SyncIndexSummary,
    sync_index::SyncIndexedFileKind,
    sync_model::{
        SyncCrdtEnvelope, SyncDeviceInfo, SyncDocumentRecord, SyncFileStatus, SyncRoot,
        normalize_home_relative_path,
    },
    sync_server::SyncObjectManifest,
};

pub const DEFAULT_SYNC_FILE_PAGE_LIMIT: usize = 100;
pub const MAX_SYNC_FILE_PAGE_LIMIT: usize = 500;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncStatusResponse {
    pub device: SyncDeviceInfo,
    #[serde(default)]
    pub connected_devices: Vec<SyncDeviceInfo>,
    pub default_root: String,
    pub roots: Vec<SyncRoot>,
    pub files: Vec<SyncDocumentRecord>,
    pub local_index: SyncIndexSummary,
    pub websocket: SyncTransportSummary,
    pub finder_state_path: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncTransportSummary {
    pub mode: String,
    pub endpoint: String,
    pub connected: bool,
}

impl Default for SyncTransportSummary {
    fn default() -> Self {
        Self {
            mode: "websocket".to_string(),
            endpoint: "/api/sync/ws".to_string(),
            connected: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncRootRequest {
    pub alias: String,
    pub relative_path: String,
    pub space_id: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncFilesQuery {
    pub space_id: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}

impl SyncFilesQuery {
    pub fn space_id(&self) -> String {
        self.space_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("main")
            .to_string()
    }

    pub fn normalized_cursor(&self) -> SyncResult<Option<String>> {
        self.cursor
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(normalize_home_relative_path)
            .transpose()
    }

    pub fn normalized_limit(&self) -> usize {
        self.limit
            .unwrap_or(DEFAULT_SYNC_FILE_PAGE_LIMIT)
            .clamp(1, MAX_SYNC_FILE_PAGE_LIMIT)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncFilesResponse {
    pub space_id: String,
    pub files: Vec<SyncFileListItem>,
    pub next_cursor: Option<String>,
    pub limit: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncFileListItem {
    pub space_id: String,
    pub relative_path: String,
    pub file_kind: SyncIndexedFileKind,
    pub content_hash: String,
    pub crdt_version: Vec<u8>,
    pub status: SyncFileStatus,
    pub size_bytes: Option<u64>,
    pub updated_by_device: String,
}

impl SyncFileListItem {
    pub fn from_document(space_id: impl Into<String>, document: &SyncDocumentRecord) -> Self {
        Self {
            space_id: space_id.into(),
            relative_path: document.relative_path.clone(),
            file_kind: SyncIndexedFileKind::Text,
            content_hash: document.content_hash.clone(),
            crdt_version: document.crdt_version.clone(),
            status: document.status,
            size_bytes: None,
            updated_by_device: document.device_name.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncApplyTextRequest {
    pub relative_path: String,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncDeleteTextRequest {
    pub relative_path: String,
    pub unicode_index: usize,
    pub unicode_len: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncApplyTextResponse {
    pub file: SyncDocumentRecord,
    pub update: SyncCrdtEnvelope,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncImportUpdateRequest {
    pub envelope: SyncCrdtEnvelope,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncImportUpdateResponse {
    pub file: SyncDocumentRecord,
    pub complete: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SyncWireMessage {
    Hello {
        device: SyncDeviceInfo,
        roots: Vec<SyncRoot>,
    },
    Update {
        envelope: SyncCrdtEnvelope,
    },
    Ack {
        relative_path: String,
        version: Vec<u8>,
    },
    RequestSnapshot {
        relative_path: String,
    },
    Snapshot {
        envelope: SyncCrdtEnvelope,
    },
    ObjectManifest {
        manifest: SyncObjectManifest,
        source_device: String,
    },
    FileDeleted {
        relative_path: String,
        source_device: String,
    },
    Heartbeat {
        device_name: String,
    },
    Error {
        message: String,
    },
}
