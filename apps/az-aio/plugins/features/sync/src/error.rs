#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use thiserror::Error;

pub type SyncResult<T> = Result<T, SyncError>;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("path `{path:?}` is outside home directory `{home:?}`")]
    PathOutsideHome { path: PathBuf, home: PathBuf },
    #[error("sync document `{relative_path}` does not exist")]
    MissingDocument { relative_path: String },
    #[error("invalid sync relative path `{value}`")]
    InvalidRelativePath { value: String },
    #[error("invalid sync file kind `{value}`")]
    InvalidFileKind { value: String },
    #[error("invalid sync file status `{value}`")]
    InvalidFileStatus { value: String },
    #[error(
        "object hash mismatch for `{relative_path}`: expected `{expected}`, restored `{actual}`"
    )]
    ObjectHashMismatch {
        relative_path: String,
        expected: String,
        actual: String,
    },
    #[error("local index path `{path:?}` is inside sync root `{root:?}`")]
    IndexInsideSyncRoot { path: PathBuf, root: PathBuf },
    #[error("unauthorized sync WebSocket connection")]
    UnauthorizedWebSocket,
    #[error("sync WebSocket upgrade failed: {0}")]
    WebSocketUpgrade(String),
    #[error("sync WebSocket transport failed: {0}")]
    WebSocketTransport(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("invalid sync WebSocket auth header: {0}")]
    WebSocketAuthHeader(String),
    #[error("sync wire JSON failed: {0}")]
    WireJson(#[source] serde_json::Error),
    #[error("file watcher {operation} failed")]
    Watch {
        operation: &'static str,
        #[source]
        source: notify::Error,
    },
    #[error("CRDT {operation} failed")]
    Crdt {
        operation: &'static str,
        #[source]
        source: az_line_crdt::LineCrdtError,
    },
    #[error("I/O failed for `{path:?}`")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("JSON failed for `{path:?}`")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("object storage failed")]
    ObjectStorage(#[from] az_rustfs::StorageError),
    #[error("PostgreSQL sync repository failed")]
    Sqlx(#[from] sqlx::Error),
}

impl SyncError {
    pub(crate) fn crdt(operation: &'static str, source: az_line_crdt::LineCrdtError) -> Self {
        Self::Crdt { operation, source }
    }
}
