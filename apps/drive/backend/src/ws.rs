//! # WebSocket CRDT sync endpoint with peer broadcast
//!
//! Accepts az-aio web client connections on `/ws/sync`. Each connection
//! receives a unique `peer_id`. When one peer pushes a CRDT update, the
//! server persists it and broadcasts the update to every other peer
//! watching the same `remote_path`.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Context;
use az_crdt::document::LineCrdtDocument;
use az_drive_core::api::{EntryKey, RelativePath, RootAlias, content_hash, object_key_for_hash};
use az_drive_store::api::{DriveEntryKind, DriveMetadataStore, DriveObjectStore};
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
    Open { remote_path: String },
    Snapshot { remote_path: String, snapshot: String, version: String },
    Update { remote_path: String, update: String, base_version: Option<String> },
    Close { remote_path: String },
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
        self.metadata.upsert_entry(&key, DriveEntryKind::File).await?;
        Ok(hash)
    }

    // ── external trigger from DriveAgent ─────────────────────────────

    /// Called when the local DriveAgent detects a file change.
    /// Re-loads the document from the store, computes a CRDT delta,
    /// and broadcasts it to every connected peer.
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
                            CrdtSyncMsg::Open { remote_path } => {
                                if let Err(err) = handle_open(
                                    &mut ws_tx, &state, &remote_path, peer_id,
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

async fn handle_open(
    ws_tx: &mut (impl SinkExt<Message, Error = axum::Error> + Unpin),
    state: &CrdtSyncState,
    remote_path: &str,
    peer_id: u64,
    cmd_tx: &mpsc::UnboundedSender<PeerCmd>,
    watched: &mut Vec<String>,
) -> anyhow::Result<()> {
    state.register_peer(remote_path, PeerHandle { cmd_tx: cmd_tx.clone() }).await;

    let (snapshot_b64, version_b64) = {
        let mut docs = state.docs.lock().await;
        let entry = if let Some(Some(entry)) = docs.get_mut(remote_path) {
            entry
        } else {
            let doc = state.load_doc(remote_path, peer_id).await?;
            let version = doc.version_bytes();
            let hash = content_hash(doc.text().as_bytes());
            docs.insert(remote_path.to_owned(), Some(DocEntry { doc, version: version.clone(), hash }));
            docs.get_mut(remote_path).unwrap().as_mut().unwrap()
        };
        let snapshot = entry.doc.export_snapshot()?;
        let version = entry.doc.version();
        (base64(snapshot.as_bytes()), base64(version.as_bytes()))
    };

    let msg = serde_json::to_string(&CrdtSyncMsg::Snapshot {
        remote_path: remote_path.to_owned(),
        snapshot: snapshot_b64,
        version: version_b64,
    })?;
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
