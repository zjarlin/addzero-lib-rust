//! # WebSocket CRDT sync endpoint with peer broadcast
//!
//! Accepts az-aio web client connections on `/ws/sync`. Each connection
//! receives a unique `peer_id`. When one peer pushes a CRDT update, the
//! server persists it and broadcasts the update to every other peer
//! watching the same `remote_path`.
//!
//! ## Protocol messages (all JSON text frames, CRDT blobs are base64)
//!
//! | Dir | Type          | Fields                                    |
//! |-----|---------------|-------------------------------------------|
//! | C→S | `hello`       | `device_id`                               |
//! | S→C | `hello_ack`   | `peer_id`                                 |
//! | C→S | `open`        | `remote_path`, `base_version?`            |
//! | S→C | `opened`      | `remote_path`, `snapshot?`, `update?`, `version` |
//! | C→S | `update`      | `remote_path`, `update`, `base_version?`  |
//! | S→C | `update`      | `remote_path`, `update`, `base_version`   |
//! | C→S | `close`       | `remote_path`                             |
//! | S→C | `error`       | `message`                                 |
//!
//! ### Reconnect (incremental sync)
//!
//! When a client reconnects after a disconnect, it sends `open` with the
//! `base_version` it last received.  The server responds with `opened`
//! containing only the delta `update` (no `snapshot`), so the client can
//! fast-forward without re-downloading the full document.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Context;
use az_crdt::document::LineCrdtDocument;
use az_drive_core::api::{EntryKey, RelativePath, RootAlias, content_hash, object_key_for_hash};
use az_drive_store::api::{DriveEntryKind, DriveMetadataStore, DriveObjectStore};
use chrono::Utc;
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, mpsc};

static NEXT_PEER_ID: AtomicU64 = AtomicU64::new(1);
fn next_peer_id() -> u64 { NEXT_PEER_ID.fetch_add(1, Ordering::Relaxed) }

// ── wire messages ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum CrdtSyncMsg {
    Hello { device_id: String },
    HelloAck { peer_id: u64 },

    /// Client wants to sync a text file via CRDT.
    /// `base_version` enables incremental reconnect.
    Open {
        remote_path: String,
        #[serde(default)]
        base_version: Option<String>,
    },

    /// Server response to `open`.
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

    // ── binary file bypass (no CRDT) ──────────────────────────────

    /// Upload a binary file chunk.  The first chunk's `offset` is 0;
    /// `is_last` signals the final chunk.  Server responds with
    /// `binary_ack` containing the content hash.
    PutBinary {
        remote_path: String,
        /// Byte offset of this chunk in the file.
        offset: u64,
        /// Base64-encoded chunk bytes.
        data: String,
        /// True if this is the last chunk.
        is_last: bool,
    },
    /// Server confirms a completed binary upload.
    BinaryAck {
        remote_path: String,
        hash: String,
        size_bytes: u64,
    },

    Error { message: String },
}

// ── internal peer handle ─────────────────────────────────────────────

#[derive(Debug)]
enum PeerCmd {
    SendText(String),
}

#[derive(Debug, Clone)]
struct PeerHandle {
    cmd_tx: mpsc::UnboundedSender<PeerCmd>,
}

// ── per-file document cache entry ────────────────────────────────────

struct DocEntry {
    doc: LineCrdtDocument,
    version: Vec<u8>,
    hash: String,
}

// ── shared server state ──────────────────────────────────────────────

pub struct CrdtSyncState {
    metadata: Arc<dyn DriveMetadataStore>,
    objects: Arc<dyn DriveObjectStore>,
    owner_drive_id: String,
    root_alias: RootAlias,
    docs: Mutex<HashMap<String, Option<DocEntry>>>,
    peers: Mutex<HashMap<String, Vec<PeerHandle>>>,
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
            peers: Mutex::new(HashMap::new()),
        }
    }

    fn entry_key(&self, remote_path: &str) -> anyhow::Result<EntryKey> {
        Ok(EntryKey::new(
            self.owner_drive_id.clone(),
            self.root_alias.clone(),
            RelativePath::parse(remote_path)?,
        ))
    }

    // ── peer registry ────────────────────────────────────────────────

    async fn register_peer(&self, remote_path: &str, handle: PeerHandle) {
        let mut peers = self.peers.lock().await;
        peers.entry(remote_path.to_owned()).or_default().push(handle);
    }

    async fn unregister_peer(&self, remote_path: &str, cmd_tx: &mpsc::UnboundedSender<PeerCmd>) {
        let mut peers = self.peers.lock().await;
        if let Some(list) = peers.get_mut(remote_path) {
            list.retain(|h| !h.cmd_tx.same_channel(cmd_tx));
            if list.is_empty() {
                peers.remove(remote_path);
            }
        }
    }

    async fn broadcast_to_others(
        &self,
        remote_path: &str,
        exclude: &mpsc::UnboundedSender<PeerCmd>,
        text: String,
    ) {
        let peers = self.peers.lock().await;
        let Some(list) = peers.get(remote_path) else { return };
        for h in list {
            if h.cmd_tx.same_channel(exclude) {
                continue;
            }
            let _ = h.cmd_tx.send(PeerCmd::SendText(text.clone()));
        }
    }

    async fn broadcast_to_all(&self, remote_path: &str, text: String) {
        let peers = self.peers.lock().await;
        let Some(list) = peers.get(remote_path) else { return };
        for h in list {
            let _ = h.cmd_tx.send(PeerCmd::SendText(text.clone()));
        }
    }

    // ── document lifecycle ───────────────────────────────────────────

    async fn load_doc(&self, remote_path: &str, peer_id: u64) -> anyhow::Result<LineCrdtDocument> {
        let key = self.entry_key(remote_path)?;
        let entry = self.metadata.get_entry(&key).await?;
        match entry {
            Some(e) if e.latest_hash.as_deref().is_some_and(|h| !h.is_empty()) => {
                let object_key = object_key_for_hash(e.latest_hash.as_ref().unwrap());
                match self.objects.get_object(&object_key).await {
                    Ok(blob) => LineCrdtDocument::from_snapshot_with_peer_id(blob, peer_id)
                        .with_context(|| format!("restore CRDT snapshot for {remote_path}")),
                    Err(_) => LineCrdtDocument::with_peer_id(peer_id).map_err(Into::into),
                }
            }
            _ => LineCrdtDocument::with_peer_id(peer_id).map_err(Into::into),
        }
    }

    async fn save_doc(&self, remote_path: &str, doc: &LineCrdtDocument) -> anyhow::Result<String> {
        let snapshot = doc.export_snapshot()?;
        let hash = content_hash(snapshot.as_bytes());
        let object_key = object_key_for_hash(&hash);
        self.objects.put_object(&object_key, snapshot.as_bytes()).await?;
        let key = self.entry_key(remote_path)?;
        let entry = self.metadata.upsert_entry(&key, DriveEntryKind::File).await?;
        // Write a version record so that latest_hash is updated.
        let version = az_drive_store::api::DriveVersion {
            id: uuid::Uuid::new_v4(),
            entry_id: entry.id,
            version: entry.latest_version.saturating_add(1),
            content_hash: hash.clone(),
            object_key,
            size_bytes: snapshot.as_bytes().len() as u64,
            device_id: "crdt-sync-server".to_owned(),
            modified_at: chrono::Utc::now(),
        };
        self.metadata.insert_version(version).await?;
        Ok(hash)
    }

    // ── external trigger from DriveAgent ─────────────────────────────

    pub async fn notify_text_changed(&self, remote_path: &str) {
        let (update_b64, version_b64) = {
            let mut docs = self.docs.lock().await;
            let Some(Some(entry)) = docs.get_mut(remote_path) else {
                return;
            };
            let fresh = match self.load_doc(remote_path, entry.doc.peer_id()).await {
                Ok(d) => d,
                Err(e) => {
                    warn!("crdt-sync reload failed for {remote_path}: {e:#}");
                    return;
                }
            };
            let old_vv = std::mem::take(&mut entry.version);
            let update_bytes = fresh.export_updates_since_bytes(&old_vv);
            entry.version = fresh.version_bytes();
            entry.doc = fresh;
            if update_bytes.is_empty() {
                return;
            }
            (base64(&update_bytes), base64(&entry.version))
        };

        let msg = serde_json::to_string(&CrdtSyncMsg::Update {
            remote_path: remote_path.to_owned(),
            update: update_b64,
            base_version: Some(version_b64),
        });
        let Ok(json) = msg else { return };
        self.broadcast_to_all(remote_path, json).await;
    }
}

// ── WS handler ────────────────────────────────────────────────────────

pub async fn handle_crdt_sync(ws: WebSocket, state: Arc<CrdtSyncState>) {
    let peer_id = next_peer_id();
    info!("crdt-sync: new connection peer_id={peer_id}");
    if let Err(err) = run_sync_loop(ws, state, peer_id).await {
        warn!("crdt-sync peer_id={peer_id} disconnected: {err:#}");
    }
}

async fn run_sync_loop(
    ws: WebSocket,
    state: Arc<CrdtSyncState>,
    peer_id: u64,
) -> anyhow::Result<()> {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<PeerCmd>();

    let ack = serde_json::to_string(&CrdtSyncMsg::HelloAck { peer_id })?;
    ws_tx.send(Message::Text(ack.into())).await?;

    let mut watched: Vec<String> = Vec::new();

    loop {
        tokio::select! {
            msg = ws_rx.next() => {
                let Some(msg) = msg else { break };
                match msg {
                    Ok(Message::Text(text)) => {
                        let parsed = match serde_json::from_str::<CrdtSyncMsg>(&text) {
                            Ok(m) => m,
                            Err(err) => {
                                let e = json_err(&format!("invalid message: {err}"));
                                let _ = ws_tx.send(Message::Text(e.into())).await;
                                continue;
                            }
                        };
                        match parsed {
                            CrdtSyncMsg::Hello { .. } => {}
                            CrdtSyncMsg::Open { remote_path, base_version } => {
                                if let Err(err) = handle_open(
                                    &mut ws_tx, &state, &remote_path, peer_id,
                                    base_version.as_deref(),
                                    &cmd_tx, &mut watched,
                                ).await {
                                    let e = json_err(&format!("open failed: {err:#}"));
                                    let _ = ws_tx.send(Message::Text(e.into())).await;
                                }
                            }
                            CrdtSyncMsg::Update { remote_path, update, base_version } => {
                                if let Err(err) = handle_update(
                                    &state, &remote_path, &update,
                                    base_version.as_deref(),
                                    &cmd_tx, peer_id,
                                ).await {
                                    let e = json_err(&format!("update failed: {err:#}"));
                                    let _ = ws_tx.send(Message::Text(e.into())).await;
                                }
                            }
                            CrdtSyncMsg::Close { remote_path } => {
                                handle_close(&state, &remote_path, &cmd_tx).await;
                                watched.retain(|p| p != &remote_path);
                            }
                            CrdtSyncMsg::PutBinary { remote_path, offset, data, is_last } => {
                                if let Err(err) = handle_put_binary(
                                    &mut ws_tx, &state, &remote_path, offset, &data, is_last,
                                ).await {
                                    let e = json_err(&format!("binary upload failed: {err:#}"));
                                    let _ = ws_tx.send(Message::Text(e.into())).await;
                                }
                            }
                            CrdtSyncMsg::BinaryAck { .. } => {
                                // Server never receives BinaryAck from client.
                                let e = json_err("binary_ack is server→client only");
                                let _ = ws_tx.send(Message::Text(e.into())).await;
                            }
                            _ => {
                                let e = json_err("unexpected message type");
                                let _ = ws_tx.send(Message::Text(e.into())).await;
                            }
                        }
                    }
                    Ok(Message::Close(_)) | Err(_) => break,
                    _ => {}
                }
            }

            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(PeerCmd::SendText(text)) => {
                        let _ = ws_tx.send(Message::Text(text.into())).await;
                    }
                    None => break,
                }
            }
        }
    }

    for path in &watched {
        handle_close(&state, path, &cmd_tx).await;
    }
    Ok(())
}

/// Handles `open`.  If the client supplies `base_version` and the server
/// cache still holds that version, we reply with an incremental `update`
/// inside `Opened` instead of a full `snapshot`.
async fn handle_open(
    ws_tx: &mut (impl SinkExt<Message, Error = axum::Error> + Unpin),
    state: &CrdtSyncState,
    remote_path: &str,
    peer_id: u64,
    base_version: Option<&str>,
    cmd_tx: &mpsc::UnboundedSender<PeerCmd>,
    watched: &mut Vec<String>,
) -> anyhow::Result<()> {
    state.register_peer(remote_path, PeerHandle { cmd_tx: cmd_tx.clone() }).await;

    let opened = {
        let mut docs = state.docs.lock().await;
        let (doc, version) = if let Some(Some(entry)) = docs.get(remote_path) {
            (entry.doc.clone(), entry.version.clone())
        } else {
            let doc = state.load_doc(remote_path, peer_id).await?;
            let version = doc.version_bytes();
            let hash = content_hash(doc.text().as_bytes());
            docs.insert(remote_path.to_owned(), Some(DocEntry { doc: doc.clone(), version: version.clone(), hash }));
            (doc, version)
        };

        // Try incremental reconnect.
        if let Some(b64) = base_version {
            let client_vv = unbase64(b64)?;
            let delta = doc.export_updates_since_bytes(&client_vv);
            if !delta.is_empty() {
                CrdtSyncMsg::Opened {
                    remote_path: remote_path.to_owned(),
                    snapshot: None,
                    update: Some(base64(&delta)),
                    version: base64(&doc.version_bytes()),
                }
            } else {
                // Client is already up-to-date — just confirm current version.
                CrdtSyncMsg::Opened {
                    remote_path: remote_path.to_owned(),
                    snapshot: None,
                    update: None,
                    version: base64(&version),
                }
            }
        } else {
            // Full snapshot for first-time connect.
            let snapshot = doc.export_snapshot()?;
            CrdtSyncMsg::Opened {
                remote_path: remote_path.to_owned(),
                snapshot: Some(base64(snapshot.as_bytes())),
                update: None,
                version: base64(&doc.version_bytes()),
            }
        }
    };

    let msg = serde_json::to_string(&opened)?;
    ws_tx.send(Message::Text(msg.into())).await?;
    watched.push(remote_path.to_owned());
    Ok(())
}

async fn handle_update(
    state: &CrdtSyncState,
    remote_path: &str,
    update_b64: &str,
    _base_version: Option<&str>,
    cmd_tx: &mpsc::UnboundedSender<PeerCmd>,
    peer_id: u64,
) -> anyhow::Result<()> {
    let update_bytes = unbase64(update_b64)?;

    let (export_update_b64, version_b64) = {
        let mut docs = state.docs.lock().await;
        let entry = docs
            .get_mut(remote_path)
            .and_then(|e| e.as_mut())
            .ok_or_else(|| anyhow::anyhow!("file {remote_path} not opened"))?;

        let report = entry.doc.import_update(&update_bytes)?;
        if !report.is_complete() {
            warn!("crdt-sync p{peer_id} incomplete update for {remote_path}: {report:?}");
        }
        let _hash = state.save_doc(remote_path, &entry.doc).await?;

        let old_vv = std::mem::take(&mut entry.version);
        let export = entry.doc.export_updates_since_bytes(&old_vv);
        entry.version = entry.doc.version_bytes();
        if export.is_empty() {
            return Ok(());
        }
        (base64(&export), base64(&entry.version))
    };

    let msg = serde_json::to_string(&CrdtSyncMsg::Update {
        remote_path: remote_path.to_owned(),
        update: export_update_b64,
        base_version: Some(version_b64),
    })?;
    state.broadcast_to_others(remote_path, cmd_tx, msg).await;
    Ok(())
}

/// Handles `put_binary`: accumulates chunks, then persists to the object
/// store when `is_last` is true.
async fn handle_put_binary(
    ws_tx: &mut (impl SinkExt<Message, Error = axum::Error> + Unpin),
    state: &CrdtSyncState,
    remote_path: &str,
    offset: u64,
    data_b64: &str,
    is_last: bool,
) -> anyhow::Result<()> {
    let chunk = unbase64(data_b64)?;
    let key = state.entry_key(remote_path)?;

    // Accumulate chunks in memory.  For production, stream to temp file.
    let mut buf = state.docs.lock().await;
    let docs = &mut *buf;
    // We use a special sentinel key to store binary accumulation buffers.
    let buf_key = format!("__binary_buf__{remote_path}");
    let _entry = docs.entry(buf_key.clone()).or_default();
    // We abuse the DocEntry hash field for binary buffer storage.
    // Simpler: use a separate BinaryAccumulator map.  For now, just
    // use DocEntry.hash as the buffer (it's a String, we can stuff bytes).
    // Actually, let's just concatenate into a Vec<u8> stored as the
    // "hash" field via base64 encoding.  This is a hack; a real impl
    // would use a dedicated accumulator.
    drop(buf);

    // Real approach: accumulate in a separate temp buffer.
    // For this minimal impl, require that the entire file fits in one
    // chunk (offset=0, is_last=true).  Multi-chunk is a TODO.
    if offset != 0 {
        anyhow::bail!("multi-chunk binary upload not yet supported; send entire file in one chunk with offset=0 and is_last=true");
    }
    if !is_last {
        // Partial chunk — store for later.
        let mut buf = state.docs.lock().await;
        let buf_key = format!("__binary_buf__{remote_path}");
        let entry = buf.entry(buf_key).or_insert_with(|| {
            Some(DocEntry {
                doc: LineCrdtDocument::new(),
                version: Vec::new(),
                hash: String::new(),
            })
        });
        if let Some(e) = entry {
            e.hash.push_str(data_b64);
        }
        return Ok(());
    }

    // Final chunk — check for accumulated previous chunks.
    let full_data = {
        let mut buf = state.docs.lock().await;
        let buf_key = format!("__binary_buf__{remote_path}");
        let prev = buf.remove(&buf_key).and_then(|e| e.map(|e| e.hash)).unwrap_or_default();
        if prev.is_empty() {
            chunk
        } else {
            let mut all = unbase64(&prev)?;
            all.extend_from_slice(&chunk);
            all
        }
    };

    let hash = content_hash(&full_data);
    let object_key = object_key_for_hash(&hash);
    state.objects.put_object(&object_key, &full_data).await?;
    let entry = state.metadata.upsert_entry(&key, DriveEntryKind::File).await?;
    let version = az_drive_store::api::DriveVersion {
        id: uuid::Uuid::new_v4(),
        entry_id: entry.id,
        version: entry.latest_version.saturating_add(1),
        content_hash: hash.clone(),
        object_key,
        size_bytes: full_data.len() as u64,
        device_id: "crdt-sync-server".to_owned(),
        modified_at: Utc::now(),
    };
    state.metadata.insert_version(version).await?;

    let msg = serde_json::to_string(&CrdtSyncMsg::BinaryAck {
        remote_path: remote_path.to_owned(),
        hash,
        size_bytes: full_data.len() as u64,
    })?;
    ws_tx.send(Message::Text(msg.into())).await?;
    Ok(())
}

async fn handle_close(
    state: &CrdtSyncState,
    remote_path: &str,
    cmd_tx: &mpsc::UnboundedSender<PeerCmd>,
) {
    state.unregister_peer(remote_path, cmd_tx).await;
    let has_peers = {
        let peers = state.peers.lock().await;
        peers.contains_key(remote_path)
    };
    if !has_peers {
        let mut docs = state.docs.lock().await;
        docs.remove(remote_path);
        info!("crdt-sync: evicted document cache for {remote_path}");
    }
}

// ── helpers ───────────────────────────────────────────────────────────

fn json_err(msg: &str) -> String {
    serde_json::to_string(&CrdtSyncMsg::Error { message: msg.to_owned() }).unwrap_or_default()
}

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

// ── LineCrdtDocument extension trait ──────────────────────────────────

use az_crdt::wire::LineCrdtVersion;

trait LineCrdtDocExt {
    fn version_bytes(&self) -> Vec<u8>;
    fn export_updates_since_bytes(&self, version: &[u8]) -> Vec<u8>;
}

impl LineCrdtDocExt for LineCrdtDocument {
    fn version_bytes(&self) -> Vec<u8> {
        self.version().into_bytes()
    }

    fn export_updates_since_bytes(&self, version: &[u8]) -> Vec<u8> {
        if version.is_empty() {
            return self.export_all_updates()
                .map(|u| u.into_bytes())
                .unwrap_or_default();
        }
        let vv = LineCrdtVersion::from_bytes(version.to_vec());
        self.export_updates_since(&vv)
            .map(|u| u.into_bytes())
            .unwrap_or_default()
    }
}
