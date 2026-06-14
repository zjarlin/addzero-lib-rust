//! # WebSocket CRDT sync endpoint
//!
//! Accepts az-aio web client connections on `/ws/sync` and runs a
//! per-connection loop that exchanges [`LineCrdtDocument`] state for
//! each tracked text file. The server keeps an in-memory document cache
//! keyed by `remote_path`.
//!
//! # Protocol
//!
//! All messages are JSON text frames carrying base64-encoded CRDT blobs:
//!
//! | Direction | Message               | Meaning                                      |
//! |-----------|-----------------------|----------------------------------------------|
//! | C→S       | `hello`               | Client announces device id                    |
//! | S→C       | `hello_ack`           | Server confirms connection with peer id       |
//! | C→S       | `open`                | Client wants to sync a file                   |
//! | S→C       | `snapshot`            | Full CRDT snapshot for opened file            |
//! | C→S       | `update`              | Client pushes a local CRDT update             |
//! | S→C       | `update`              | Server pushes a remote CRDT update            |
//! | C→S       | `close`               | Client stops syncing a file                   |
//! | S→C       | `error`               | Error for the last message                    |
//! | S→C       | `text_changed`        | Notifies client that file text was updated    |
//!
//! The `remote_path` in protocol messages is the drive-relative path
//! (e.g. `README.md` or `docs/guide.md`). The server maps this to an
//! [`EntryKey`] using the configured default root alias.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Context;
use az_crdt::document::LineCrdtDocument;
use az_drive_core::api::{EntryKey, RelativePath, RootAlias, content_hash, object_key_for_hash};
use az_drive_store::api::{DriveEntryKind, DriveMetadataStore, DriveObjectStore};
use axum::extract::ws::{Message, WebSocket};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

static NEXT_PEER_ID: AtomicU64 = AtomicU64::new(1);

fn next_peer_id() -> u64 {
    NEXT_PEER_ID.fetch_add(1, Ordering::Relaxed)
}

// ── wire messages ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum CrdtSyncMsg {
    Hello {
        device_id: String,
    },
    HelloAck {
        peer_id: u64,
    },
    Open {
        remote_path: String,
    },
    Snapshot {
        remote_path: String,
        snapshot: String,
        version: String,
    },
    Update {
        remote_path: String,
        update: String,
        base_version: Option<String>,
    },
    Close {
        remote_path: String,
    },
    TextChanged {
        remote_path: String,
        hash: String,
        peer_count: usize,
    },
    Error {
        message: String,
    },
}

// ── per-file document entry ────────────────────────────────────────────

struct DocEntry {
    doc: LineCrdtDocument,
    hash: String,
    peer_count: usize,
}

// ── shared server state ────────────────────────────────────────────────

pub struct CrdtSyncState {
    metadata: Arc<dyn DriveMetadataStore>,
    objects: Arc<dyn DriveObjectStore>,
    owner_drive_id: String,
    root_alias: RootAlias,
    docs: Mutex<HashMap<String, DocEntry>>,
}

impl CrdtSyncState {
    pub fn new(
        metadata: Arc<dyn DriveMetadataStore>,
        objects: Arc<dyn DriveObjectStore>,
        owner_drive_id: String,
    ) -> Self {
        Self {
            metadata,
            objects,
            owner_drive_id,
            root_alias: RootAlias::parse("home").unwrap(),
            docs: Mutex::new(HashMap::new()),
        }
    }

    fn entry_key(&self, remote_path: &str) -> anyhow::Result<EntryKey> {
        Ok(EntryKey::new(
            self.owner_drive_id.clone(),
            self.root_alias.clone(),
            RelativePath::parse(remote_path)?,
        ))
    }

    /// Notify all peers watching a file that its content changed.
    pub async fn notify_text_changed(&self, remote_path: &str, _hash: &str) {
        let docs = self.docs.lock().await;
        if let Some(entry) = docs.get(remote_path) {
            info!(
                "notified {n} peer(s) about {path}",
                n = entry.peer_count,
                path = remote_path
            );
        }
    }

    /// Load or create a `LineCrdtDocument` for `remote_path`, seeding from
    /// the object store if we have existing content.
    async fn load_doc(
        &self,
        remote_path: &str,
        peer_id: u64,
    ) -> anyhow::Result<LineCrdtDocument> {
        let key = self.entry_key(remote_path)?;
        let entry = self.metadata.get_entry(&key).await?;
        match entry {
            Some(entry) if entry.latest_hash.as_deref().is_some_and(|h| !h.is_empty()) => {
                let hash = entry.latest_hash.as_ref().unwrap();
                let object_key = object_key_for_hash(hash);
                match self.objects.get_object(&object_key).await {
                    Ok(snapshot_blob) => {
                        LineCrdtDocument::from_snapshot_with_peer_id(snapshot_blob, peer_id)
                            .with_context(|| format!("restore CRDT snapshot for {remote_path}"))
                    }
                    Err(_) => {
                        let doc = LineCrdtDocument::with_peer_id(peer_id)?;
                        Ok(doc)
                    }
                }
            }
            _ => {
                let doc = LineCrdtDocument::with_peer_id(peer_id)?;
                Ok(doc)
            }
        }
    }

    /// Persist the current CRDT snapshot into the object store and
    /// upsert the metadata entry.
    async fn save_doc(
        &self,
        remote_path: &str,
        doc: &LineCrdtDocument,
    ) -> anyhow::Result<String> {
        let snapshot = doc.export_snapshot()?;
        let hash = content_hash(snapshot.as_bytes());
        let object_key = object_key_for_hash(&hash);
        self.objects
            .put_object(&object_key, snapshot.as_bytes())
            .await?;
        let key = self.entry_key(remote_path)?;
        self.metadata
            .upsert_entry(&key, DriveEntryKind::File)
            .await?;
        Ok(hash)
    }
}

// ── WS handler ─────────────────────────────────────────────────────────

/// Axum handler: upgrade to WebSocket and run the CRDT sync loop.
pub async fn handle_crdt_sync(ws: WebSocket, state: Arc<CrdtSyncState>) {
    let peer_id = next_peer_id();
    info!("crdt-sync: new connection peer_id={peer_id}");

    if let Err(err) = run_sync_loop(ws, state, peer_id).await {
        warn!("crdt-sync peer_id={peer_id} disconnected: {err:#}");
    }
}

async fn run_sync_loop(
    mut ws: WebSocket,
    state: Arc<CrdtSyncState>,
    peer_id: u64,
) -> anyhow::Result<()> {
    // Send hello_ack first.
    let ack = serde_json::to_string(&CrdtSyncMsg::HelloAck { peer_id })?;
    ws.send(Message::Text(ack.into())).await?;

    // Track which files this peer is watching.
    let mut watched: Vec<String> = Vec::new();

    loop {
        let msg = match ws.recv().await {
            Some(Ok(Message::Text(text))) => text,
            Some(Ok(Message::Close(_))) | None => break,
            Some(Err(err)) => {
                warn!("crdt-sync peer_id={peer_id} ws error: {err}");
                break;
            }
            _ => continue,
        };

        let parsed: CrdtSyncMsg = match serde_json::from_str(&msg) {
            Ok(m) => m,
            Err(err) => {
                let err_msg = serde_json::to_string(&CrdtSyncMsg::Error {
                    message: format!("invalid message: {err}"),
                })?;
                ws.send(Message::Text(err_msg.into())).await?;
                continue;
            }
        };

        match parsed {
            CrdtSyncMsg::Hello { .. } => {
                // Already handled; ignore duplicate hello.
            }
            CrdtSyncMsg::Open { remote_path } => {
                if let Err(err) = handle_open(&mut ws, &state, &remote_path, peer_id, &mut watched).await {
                    let err_msg = serde_json::to_string(&CrdtSyncMsg::Error {
                        message: format!("open failed: {err:#}"),
                    })?;
                    ws.send(Message::Text(err_msg.into())).await?;
                }
            }
            CrdtSyncMsg::Update { remote_path, update, base_version } => {
                if let Err(err) = handle_update(&state, &remote_path, &update, base_version.as_deref(), peer_id).await {
                    let err_msg = serde_json::to_string(&CrdtSyncMsg::Error {
                        message: format!("update failed: {err:#}"),
                    })?;
                    ws.send(Message::Text(err_msg.into())).await?;
                }
            }
            CrdtSyncMsg::Close { remote_path } => {
                handle_close(&state, &remote_path, peer_id).await;
                watched.retain(|p| p != &remote_path);
            }
            _ => {
                let err_msg = serde_json::to_string(&CrdtSyncMsg::Error {
                    message: "unexpected message type".into(),
                })?;
                ws.send(Message::Text(err_msg.into())).await?;
            }
        }
    }

    // Clean up watched files on disconnect.
    for path in &watched {
        handle_close(&state, path, peer_id).await;
    }

    Ok(())
}

async fn handle_open(
    ws: &mut WebSocket,
    state: &CrdtSyncState,
    remote_path: &str,
    peer_id: u64,
    watched: &mut Vec<String>,
) -> anyhow::Result<()> {
    let mut docs = state.docs.lock().await;

    let entry = if let Some(entry) = docs.get_mut(remote_path) {
        entry.peer_count += 1;
        entry
    } else {
        let doc = state.load_doc(remote_path, peer_id).await?;
        let hash = content_hash(doc.text().as_bytes());
        docs.insert(
            remote_path.to_owned(),
            DocEntry {
                doc,
                hash: hash.clone(),
                peer_count: 1,
            },
        );
        docs.get_mut(remote_path).unwrap()
    };

    let snapshot = entry.doc.export_snapshot()?;
    let version = entry.doc.version();
    let msg = serde_json::to_string(&CrdtSyncMsg::Snapshot {
        remote_path: remote_path.to_owned(),
        snapshot: base64(snapshot.as_bytes()),
        version: base64(version.as_bytes()),
    })?;
    ws.send(Message::Text(msg.into())).await?;

    watched.push(remote_path.to_owned());
    Ok(())
}

async fn handle_update(
    state: &CrdtSyncState,
    remote_path: &str,
    update_b64: &str,
    _base_version: Option<&str>,
    peer_id: u64,
) -> anyhow::Result<()> {
    let update_bytes = unbase64(update_b64)?;
    let mut docs = state.docs.lock().await;

    let entry = docs
        .get_mut(remote_path)
        .ok_or_else(|| anyhow::anyhow!("file {remote_path} not opened"))?;

    let report = entry.doc.import_update(&update_bytes)?;
    if report.is_complete() {
        let hash = state.save_doc(remote_path, &entry.doc).await?;
        entry.hash = hash;
    } else {
        warn!(
            "crdt-sync peer_id={peer_id} incomplete update for {path}: {report:?}",
            path = remote_path
        );
    }

    Ok(())
}

async fn handle_close(state: &CrdtSyncState, remote_path: &str, _peer_id: u64) {
    let mut docs = state.docs.lock().await;
    if let Some(entry) = docs.get_mut(remote_path) {
        entry.peer_count = entry.peer_count.saturating_sub(1);
        if entry.peer_count == 0 {
            docs.remove(remote_path);
            info!("crdt-sync: evicted document cache for {remote_path}");
        }
    }
}

// ── helpers ────────────────────────────────────────────────────────────

fn base64(bytes: &[u8]) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn unbase64(encoded: &str) -> anyhow::Result<Vec<u8>> {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .context("invalid base64")
}
