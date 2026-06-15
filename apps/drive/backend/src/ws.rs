//! # WebSocket CRDT sync endpoint with peer broadcast
//!
//! Accepts az-aio web client connections on `/ws/sync`. Each connection
//! receives a unique `peer_id`. When one peer pushes a CRDT update, the
//! server persists it and broadcasts the update to every other peer
//! watching the same `remote_path`.
//!
//! ## Protocol messages (all JSON text frames, binary blobs are base64)
//!
//! **Text file CRDT sync:**
//! | Dir | Type      | Fields |
//! |-----|-----------|--------|
//! | C→S | `hello`   | `device_id` |
//! | S→C | `hello_ack` | `peer_id` |
//! | C→S | `open`    | `remote_path`, `base_version?` |
//! | S→C | `opened`  | `remote_path`, `snapshot?`, `update?`, `version` |
//! | C→S | `update`  | `remote_path`, `update`, `base_version?` |
//! | S→C | `update`  | `remote_path`, `update`, `base_version` |
//! | C→S | `close`   | `remote_path` |
//! | S→C | `error`   | `message` |
//!
//! **Binary file bypass (no CRDT):**
//! | Dir | Type          | Fields |
//! |-----|---------------|--------|
//! | C→S | `put_binary`  | `remote_path`, `offset`, `data`, `is_last` |
//! | S→C | `binary_ack`  | `remote_path`, `hash`, `size_bytes` |
//! | C→S | `get_binary`  | `remote_path` |
//! | S→C | `binary_chunk`| `remote_path`, `offset`, `data`, `is_last` |
//!
//! **Directory listing:**
//! | Dir | Type          | Fields |
//! |-----|---------------|--------|
//! | C→S | `list`        | `prefix?` |
//! | S→C | `list_result` | `entries: [{remote_path, kind, size_bytes, hash}]` |
//!
//! ### Incremental reconnect
//!
//! When a client reconnects, it sends `open` with the `base_version` it
//! last received. The server replies with `opened` containing only the
//! delta `update` (no `snapshot`), enabling fast-forward sync.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Context;
use az_crdt::document::LineCrdtDocument;
use az_drive_core::api::{EntryKey, RelativePath, RootAlias, content_hash, object_key_for_hash};
use az_drive_store::api::{DriveEntryKind, DriveMetadataStore, DriveObjectStore};
use axum::extract::ws::{Message, WebSocket};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::sync::mpsc::{self, UnboundedSender};

static NEXT_PEER_ID: AtomicU64 = AtomicU64::new(1);
fn next_peer_id() -> u64 { NEXT_PEER_ID.fetch_add(1, Ordering::Relaxed) }

// ══════════════════════════════════════════════════════════════════════
//  Wire messages
// ══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum CrdtSyncMsg {
    // ── connection ──────────────────────────────────────────────────
    Hello { device_id: String },
    HelloAck { peer_id: u64 },

    // ── text CRDT ───────────────────────────────────────────────────
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

    // ── binary bypass ───────────────────────────────────────────────
    PutBinary {
        remote_path: String,
        offset: u64,
        data: String,
        is_last: bool,
    },
    BinaryAck {
        remote_path: String,
        hash: String,
        size_bytes: u64,
    },
    GetBinary {
        remote_path: String,
    },
    BinaryChunk {
        remote_path: String,
        offset: u64,
        data: String,
        is_last: bool,
    },

    // ── directory listing ───────────────────────────────────────────
    List {
        #[serde(default)]
        prefix: Option<String>,
    },
    ListResult {
        entries: Vec<ListEntry>,
    },

    // ── errors ──────────────────────────────────────────────────────
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ListEntry {
    remote_path: String,
    /// `"text"` or `"binary"`
    kind: String,
    size_bytes: u64,
    hash: String,
}

// ══════════════════════════════════════════════════════════════════════
//  Internal types
// ══════════════════════════════════════════════════════════════════════

#[derive(Debug)]
enum PeerCmd {
    SendText(String),
}

#[derive(Debug, Clone)]
struct PeerHandle {
    cmd_tx: UnboundedSender<PeerCmd>,
}

struct DocEntry {
    doc: LineCrdtDocument,
    version: Vec<u8>,
}

/// Accumulates binary upload chunks before final persistence.
struct BinaryBuf {
    chunks: Vec<Vec<u8>>,
    total_len: u64,
}

// ══════════════════════════════════════════════════════════════════════
//  Shared server state
// ══════════════════════════════════════════════════════════════════════

pub struct CrdtSyncState {
    metadata: Arc<dyn DriveMetadataStore>,
    objects: Arc<dyn DriveObjectStore>,
    owner_drive_id: String,
    root_alias: RootAlias,
    /// Text document cache: `remote_path → DocEntry`.
    docs: Mutex<HashMap<String, Option<DocEntry>>>,
    /// Binary upload accumulation buffers: `remote_path → BinaryBuf`.
    binary_bufs: Mutex<HashMap<String, BinaryBuf>>,
    /// Peer registry: `remote_path → Vec<PeerHandle>`.
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
            binary_bufs: Mutex::new(HashMap::new()),
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

    async fn unregister_peer(&self, remote_path: &str, cmd_tx: &UnboundedSender<PeerCmd>) {
        let mut peers = self.peers.lock().await;
        if let Some(list) = peers.get_mut(remote_path) {
            list.retain(|h| !h.cmd_tx.same_channel(cmd_tx));
            if list.is_empty() {
                peers.remove(remote_path);
            }
        }
    }

    async fn broadcast_to_others(
        &self, remote_path: &str, exclude: &UnboundedSender<PeerCmd>, text: String,
    ) {
        let peers = self.peers.lock().await;
        let Some(list) = peers.get(remote_path) else { return };
        for h in list {
            if h.cmd_tx.same_channel(exclude) { continue; }
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
                    Err(_) => LineCrdtDocument::with_peer_id(peer_id),
                }
            }
            _ => LineCrdtDocument::with_peer_id(peer_id),
        }
    }

    async fn save_doc(&self, remote_path: &str, doc: &LineCrdtDocument) -> anyhow::Result<String> {
        let snapshot = doc.export_snapshot()?;
        let hash = content_hash(snapshot.as_bytes());
        let object_key = object_key_for_hash(&hash);
        self.objects.put_object(&object_key, snapshot.as_bytes()).await?;
        let key = self.entry_key(remote_path)?;
        let entry = self.metadata.upsert_entry(&key, DriveEntryKind::File).await?;
        let version = az_drive_store::api::DriveVersion {
            id: uuid::Uuid::new_v4(),
            entry_id: entry.id,
            version: entry.latest_version.saturating_add(1),
            content_hash: hash.clone(),
            object_key,
            size_bytes: snapshot.as_bytes().len() as u64,
            device_id: "crdt-sync-server".to_owned(),
            modified_at: Utc::now(),
        };
        self.metadata.insert_version(version).await?;
        Ok(hash)
    }

    // ── binary helpers ───────────────────────────────────────────────

    async fn save_binary(&self, remote_path: &str, data: &[u8]) -> anyhow::Result<(String, u64)> {
        let hash = content_hash(data);
        let object_key = object_key_for_hash(&hash);
        self.objects.put_object(&object_key, data).await?;
        let key = self.entry_key(remote_path)?;
        let entry = self.metadata.upsert_entry(&key, DriveEntryKind::File).await?;
        let version = az_drive_store::api::DriveVersion {
            id: uuid::Uuid::new_v4(),
            entry_id: entry.id,
            version: entry.latest_version.saturating_add(1),
            content_hash: hash.clone(),
            object_key,
            size_bytes: data.len() as u64,
            device_id: "crdt-sync-server".to_owned(),
            modified_at: Utc::now(),
        };
        self.metadata.insert_version(version).await?;
        Ok((hash, data.len() as u64))
    }

    async fn get_binary_data(&self, remote_path: &str) -> anyhow::Result<Vec<u8>> {
        let key = self.entry_key(remote_path)?;
        let entry = self.metadata.get_entry(&key).await?
            .ok_or_else(|| anyhow::anyhow!("file not found: {remote_path}"))?;
        let Some(ref hash) = entry.latest_hash else {
            anyhow::bail!("no content for {remote_path}");
        };
        let object_key = object_key_for_hash(hash);
        self.objects.get_object(&object_key).await
    }

    // ── external trigger from DriveAgent ─────────────────────────────

    pub async fn notify_text_changed(&self, remote_path: &str) {
        let (update_b64, version_b64) = {
            let mut docs = self.docs.lock().await;
            let Some(Some(entry)) = docs.get_mut(remote_path) else { return };
            let fresh = match self.load_doc(remote_path, entry.doc.peer_id()).await {
                Ok(d) => d,
                Err(e) => { warn!("crdt-sync reload failed for {remote_path}: {e:#}"); return; }
            };
            let old_vv = std::mem::take(&mut entry.version);
            let update_bytes = fresh.export_updates_since_bytes(&old_vv);
            entry.version = fresh.version_bytes();
            entry.doc = fresh;
            if update_bytes.is_empty() { return; }
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

// ══════════════════════════════════════════════════════════════════════
//  WS handler
// ══════════════════════════════════════════════════════════════════════

pub async fn handle_crdt_sync(ws: WebSocket, state: Arc<CrdtSyncState>) {
    let peer_id = next_peer_id();
    info!("crdt-sync: new connection peer_id={peer_id}");
    if let Err(err) = run_sync_loop(ws, state, peer_id).await {
        warn!("crdt-sync peer_id={peer_id} disconnected: {err:#}");
    }
}

async fn run_sync_loop(
    ws: WebSocket, state: Arc<CrdtSyncState>, peer_id: u64,
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
                                let _ = ws_tx.send(Message::Text(
                                    json_err(&format!("invalid message: {err}")).into()
                                )).await;
                                continue;
                            }
                        };
                        match parsed {
                            CrdtSyncMsg::Hello { .. } => {}
                            CrdtSyncMsg::Open { remote_path, base_version } => {
                                if let Err(err) = handle_open(
                                    &mut ws_tx, &state, &remote_path, peer_id,
                                    base_version.as_deref(), &cmd_tx, &mut watched,
                                ).await {
                                    let _ = ws_tx.send(Message::Text(
                                        json_err(&format!("open failed: {err:#}")).into()
                                    )).await;
                                }
                            }
                            CrdtSyncMsg::Update { remote_path, update, base_version } => {
                                if let Err(err) = handle_update(
                                    &state, &remote_path, &update,
                                    base_version.as_deref(), &cmd_tx, peer_id,
                                ).await {
                                    let _ = ws_tx.send(Message::Text(
                                        json_err(&format!("update failed: {err:#}")).into()
                                    )).await;
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
                                    let _ = ws_tx.send(Message::Text(
                                        json_err(&format!("binary upload failed: {err:#}")).into()
                                    )).await;
                                }
                            }
                            CrdtSyncMsg::GetBinary { remote_path } => {
                                if let Err(err) = handle_get_binary(
                                    &mut ws_tx, &state, &remote_path,
                                ).await {
                                    let _ = ws_tx.send(Message::Text(
                                        json_err(&format!("binary download failed: {err:#}")).into()
                                    )).await;
                                }
                            }
                            CrdtSyncMsg::List { prefix } => {
                                if let Err(err) = handle_list(
                                    &mut ws_tx, &state, prefix.as_deref(),
                                ).await {
                                    let _ = ws_tx.send(Message::Text(
                                        json_err(&format!("list failed: {err:#}")).into()
                                    )).await;
                                }
                            }
                            CrdtSyncMsg::HelloAck { .. }
                            | CrdtSyncMsg::BinaryAck { .. } | CrdtSyncMsg::BinaryChunk { .. }
                            | CrdtSyncMsg::Opened { .. } | CrdtSyncMsg::ListResult { .. } => {
                                let _ = ws_tx.send(Message::Text(
                                    json_err("server never receives this message type").into()
                                )).await;
                            }
                            CrdtSyncMsg::Error { .. } => {}
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

    for path in &watched { handle_close(&state, path, &cmd_tx).await; }
    Ok(())
}

// ══════════════════════════════════════════════════════════════════════
//  Message handlers
// ══════════════════════════════════════════════════════════════════════

async fn handle_open(
    ws_tx: &mut (impl SinkExt<Message, Error = axum::Error> + Unpin),
    state: &CrdtSyncState, remote_path: &str, peer_id: u64,
    base_version: Option<&str>,
    cmd_tx: &UnboundedSender<PeerCmd>, watched: &mut Vec<String>,
) -> anyhow::Result<()> {
    state.register_peer(remote_path, PeerHandle { cmd_tx: cmd_tx.clone() }).await;

    let opened = {
        let mut docs = state.docs.lock().await;
        let (doc, version) = if let Some(Some(entry)) = docs.get(remote_path) {
            (entry.doc.clone(), entry.version.clone())
        } else {
            let doc = state.load_doc(remote_path, peer_id).await?;
            let version = doc.version_bytes();
            docs.insert(remote_path.to_owned(), Some(DocEntry { doc: doc.clone(), version: version.clone() }));
            (doc, version)
        };

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
                CrdtSyncMsg::Opened {
                    remote_path: remote_path.to_owned(),
                    snapshot: None, update: None,
                    version: base64(&version),
                }
            }
        } else {
            let snapshot = doc.export_snapshot()?;
            CrdtSyncMsg::Opened {
                remote_path: remote_path.to_owned(),
                snapshot: Some(base64(snapshot.as_bytes())),
                update: None,
                version: base64(&doc.version_bytes()),
            }
        }
    };

    ws_tx.send(Message::Text(serde_json::to_string(&opened)?.into())).await?;
    watched.push(remote_path.to_owned());
    Ok(())
}

async fn handle_update(
    state: &CrdtSyncState, remote_path: &str, update_b64: &str,
    _base_version: Option<&str>,
    cmd_tx: &UnboundedSender<PeerCmd>, peer_id: u64,
) -> anyhow::Result<()> {
    let update_bytes = unbase64(update_b64)?;

    let (export_update_b64, version_b64) = {
        let mut docs = state.docs.lock().await;
        let entry = docs.get_mut(remote_path).and_then(|e| e.as_mut())
            .ok_or_else(|| anyhow::anyhow!("file {remote_path} not opened"))?;

        let report = entry.doc.import_update(&update_bytes)?;
        if !report.is_complete() {
            warn!("crdt-sync p{peer_id} incomplete update for {remote_path}: {report:?}");
        }
        let _hash = state.save_doc(remote_path, &entry.doc).await?;

        let old_vv = std::mem::take(&mut entry.version);
        let export = entry.doc.export_updates_since_bytes(&old_vv);
        entry.version = entry.doc.version_bytes();
        if export.is_empty() { return Ok(()); }
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

async fn handle_put_binary(
    ws_tx: &mut (impl SinkExt<Message, Error = axum::Error> + Unpin),
    state: &CrdtSyncState, remote_path: &str,
    _offset: u64, data_b64: &str, is_last: bool,
) -> anyhow::Result<()> {
    let chunk = unbase64(data_b64)?;
    let chunk_len = chunk.len() as u64;

    // Accumulate in binary_bufs.
    {
        let mut bufs = state.binary_bufs.lock().await;
        let buf = bufs.entry(remote_path.to_owned()).or_insert_with(|| BinaryBuf {
            chunks: Vec::new(),
            total_len: 0,
        });
        // Simple append; in production we'd verify offset == buf.total_len.
        buf.total_len += chunk_len;
        buf.chunks.push(chunk);
    }

    if !is_last {
        return Ok(()); // wait for more chunks
    }

    // Finalize: collect all chunks.
    let full_data = {
        let mut bufs = state.binary_bufs.lock().await;
        let buf = bufs.remove(remote_path).unwrap_or(BinaryBuf { chunks: Vec::new(), total_len: 0 });
        let mut all = Vec::with_capacity(buf.total_len as usize);
        for c in &buf.chunks { all.extend_from_slice(c); }
        all
    };

    if full_data.is_empty() {
        anyhow::bail!("empty binary upload");
    }

    let (hash, size) = state.save_binary(remote_path, &full_data).await?;
    let msg = serde_json::to_string(&CrdtSyncMsg::BinaryAck {
        remote_path: remote_path.to_owned(),
        hash,
        size_bytes: size,
    })?;
    ws_tx.send(Message::Text(msg.into())).await?;
    Ok(())
}

async fn handle_get_binary(
    ws_tx: &mut (impl SinkExt<Message, Error = axum::Error> + Unpin),
    state: &CrdtSyncState, remote_path: &str,
) -> anyhow::Result<()> {
    let data = state.get_binary_data(remote_path).await?;
    let total = data.len() as u64;

    // Stream in chunks (for now, single chunk; multi-chunk streaming is a TODO).
    const MAX_CHUNK: usize = 1024 * 1024; // 1 MiB
    let mut offset: u64 = 0;
    while offset < total {
        let end = std::cmp::min(offset as usize + MAX_CHUNK, total as usize);
        let chunk = &data[offset as usize..end];
        let is_last = end as u64 >= total;
        let msg = serde_json::to_string(&CrdtSyncMsg::BinaryChunk {
            remote_path: remote_path.to_owned(),
            offset,
            data: base64(chunk),
            is_last,
        })?;
        ws_tx.send(Message::Text(msg.into())).await?;
        offset = end as u64;
    }
    Ok(())
}

async fn handle_list(
    ws_tx: &mut (impl SinkExt<Message, Error = axum::Error> + Unpin),
    state: &CrdtSyncState, prefix: Option<&str>,
) -> anyhow::Result<()> {
    let prefix = prefix.unwrap_or("");
    let entries_all = state.metadata.list_entries_by_space(&state.owner_drive_id).await?;

    let entries: Vec<ListEntry> = entries_all
        .into_iter()
        .filter(|e| {
            let path = e.key.relative_path.as_str();
            path.starts_with(prefix) && !e.deleted
        })
        .map(|e| {
            let (size_bytes, hash) = match e.latest_version {
                0 => (0u64, String::new()),
                _ => {
                    // Fetch version info for size/hash.
                    // We don't have an easy way to get version by entry without
                    // another query, so use defaults.
                    (0u64, e.latest_hash.unwrap_or_default())
                }
            };
            ListEntry {
                remote_path: e.key.relative_path.as_str().to_owned(),
                kind: match e.kind {
                    DriveEntryKind::File => "file".to_owned(),
                    DriveEntryKind::Directory => "dir".to_owned(),
                },
                size_bytes,
                hash,
            }
        })
        .collect();

    let msg = serde_json::to_string(&CrdtSyncMsg::ListResult { entries })?;
    ws_tx.send(Message::Text(msg.into())).await?;
    Ok(())
}

async fn handle_close(
    state: &CrdtSyncState, remote_path: &str,
    cmd_tx: &UnboundedSender<PeerCmd>,
) {
    state.unregister_peer(remote_path, cmd_tx).await;
    let has_peers = { state.peers.lock().await.contains_key(remote_path) };
    if !has_peers {
        let mut docs = state.docs.lock().await;
        docs.remove(remote_path);
        info!("crdt-sync: evicted document cache for {remote_path}");
    }
}

// ══════════════════════════════════════════════════════════════════════
//  Helpers
// ══════════════════════════════════════════════════════════════════════

fn json_err(msg: &str) -> String {
    serde_json::to_string(&CrdtSyncMsg::Error { message: msg.to_owned() }).unwrap_or_default()
}

fn base64(bytes: &[u8]) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn unbase64(encoded: &str) -> anyhow::Result<Vec<u8>> {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.decode(encoded).context("invalid base64")
}

// ══════════════════════════════════════════════════════════════════════
//  LineCrdtDocument extensions
// ══════════════════════════════════════════════════════════════════════

use az_crdt::wire::LineCrdtVersion;

trait LineCrdtDocExt {
    fn version_bytes(&self) -> Vec<u8>;
    fn export_updates_since_bytes(&self, version: &[u8]) -> Vec<u8>;
}

impl LineCrdtDocExt for LineCrdtDocument {
    fn version_bytes(&self) -> Vec<u8> { self.version().into_bytes() }

    fn export_updates_since_bytes(&self, version: &[u8]) -> Vec<u8> {
        if version.is_empty() {
            return self.export_all_updates().map(|u| u.into_bytes()).unwrap_or_default();
        }
        let vv = LineCrdtVersion::from_bytes(version.to_vec());
        self.export_updates_since(&vv).map(|u| u.into_bytes()).unwrap_or_default()
    }
}
