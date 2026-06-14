//! Shared CRDT sync state and WebSocket handler for the Drive worker.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Context;
use axum::extract::ws::{Message, WebSocket};
use az_crdt::document::LineCrdtDocument;
use az_drive_core::api::{EntryKey, RelativePath, RootAlias, content_hash, object_key_for_hash};
use az_drive_store::api::{DriveEntryKind, DriveMetadataStore, DriveObjectStore, DriveVersion};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use log::{info, warn};
use tokio::sync::Mutex;
use tokio::sync::mpsc::{self, UnboundedSender};
use uuid::Uuid;

use crate::sync_msg::{CrdtSyncMsg, LineCrdtDocExt, b64encode, json_err, unbase64};

static NEXT_PEER_ID: AtomicU64 = AtomicU64::new(1);
fn next_peer_id() -> u64 {
    NEXT_PEER_ID.fetch_add(1, Ordering::Relaxed)
}

// ── Internal types ─────────────────────────────────────────────────────

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

// ── Shared state ───────────────────────────────────────────────────────

pub struct DriveSyncState {
    pub metadata: Arc<dyn DriveMetadataStore>,
    pub objects: Arc<dyn DriveObjectStore>,
    pub owner_drive_id: String,
    root_alias: RootAlias,
    docs: Mutex<HashMap<String, Option<DocEntry>>>,
    peers: Mutex<HashMap<String, Vec<PeerHandle>>>,
}

impl DriveSyncState {
    pub fn new(
        metadata: Arc<dyn DriveMetadataStore>,
        objects: Arc<dyn DriveObjectStore>,
        owner_drive_id: String,
    ) -> Self {
        Self {
            metadata,
            objects,
            owner_drive_id,
            root_alias: RootAlias::parse(RootAlias::HOME).unwrap(),
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

    // ── peer registry ──────────────────────────────────────────────

    async fn register_peer(&self, remote_path: &str, handle: PeerHandle) {
        self.peers
            .lock()
            .await
            .entry(remote_path.to_owned())
            .or_default()
            .push(handle);
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
        &self,
        remote_path: &str,
        exclude: &UnboundedSender<PeerCmd>,
        text: String,
    ) {
        let peers = self.peers.lock().await;
        let Some(list) = peers.get(remote_path) else {
            return;
        };
        for h in list {
            if h.cmd_tx.same_channel(exclude) {
                continue;
            }
            let _ = h.cmd_tx.send(PeerCmd::SendText(text.clone()));
        }
    }

    // ── document lifecycle ─────────────────────────────────────────

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
        let entry = self.metadata.upsert_entry(&key, DriveEntryKind::File).await?;
        let version = DriveVersion {
            id: Uuid::new_v4(),
            entry_id: entry.id,
            version: entry.latest_version.saturating_add(1),
            content_hash: hash.clone(),
            object_key,
            size_bytes: snapshot.as_bytes().len() as u64,
            device_id: "drive-worker-sync".to_owned(),
            modified_at: Utc::now(),
        };
        self.metadata.insert_version(version).await?;
        Ok(hash)
    }
}

// ── WS handler ─────────────────────────────────────────────────────────

/// Accepts a WebSocket upgrade and runs the CRDT sync protocol.
pub async fn handle_drive_sync(ws: WebSocket, state: Arc<DriveSyncState>) {
    let peer_id = next_peer_id();
    info!("drive-worker sync: new connection peer_id={peer_id}");
    if let Err(err) = run_sync_loop(ws, state, peer_id).await {
        warn!("drive-worker sync peer_id={peer_id} disconnected: {err:#}");
    }
}

async fn run_sync_loop(
    ws: WebSocket,
    state: Arc<DriveSyncState>,
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
                            CrdtSyncMsg::HelloAck { .. }
                            | CrdtSyncMsg::Opened { .. } => {
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

    for path in &watched {
        handle_close(&state, path, &cmd_tx).await;
    }
    Ok(())
}

async fn handle_open(
    ws_tx: &mut (impl SinkExt<Message, Error = axum::Error> + Unpin),
    state: &DriveSyncState,
    remote_path: &str,
    peer_id: u64,
    base_version: Option<&str>,
    cmd_tx: &UnboundedSender<PeerCmd>,
    watched: &mut Vec<String>,
) -> anyhow::Result<()> {
    state
        .register_peer(remote_path, PeerHandle {
            cmd_tx: cmd_tx.clone(),
        })
        .await;

    let opened = {
        let mut docs = state.docs.lock().await;
        let (doc, version) = if let Some(Some(entry)) = docs.get(remote_path) {
            (entry.doc.clone(), entry.version.clone())
        } else {
            let doc = state.load_doc(remote_path, peer_id).await?;
            let version = doc.version_bytes();
            docs.insert(
                remote_path.to_owned(),
                Some(DocEntry {
                    doc: doc.clone(),
                    version: version.clone(),
                }),
            );
            (doc, version)
        };

        if let Some(b64) = base_version {
            let client_vv = unbase64(b64)?;
            let delta = doc.export_updates_since_bytes(&client_vv);
            if !delta.is_empty() {
                CrdtSyncMsg::Opened {
                    remote_path: remote_path.to_owned(),
                    snapshot: None,
                    update: Some(b64encode(&delta)),
                    version: b64encode(&doc.version_bytes()),
                }
            } else {
                CrdtSyncMsg::Opened {
                    remote_path: remote_path.to_owned(),
                    snapshot: None,
                    update: None,
                    version: b64encode(&version),
                }
            }
        } else {
            let snapshot = doc.export_snapshot()?;
            CrdtSyncMsg::Opened {
                remote_path: remote_path.to_owned(),
                snapshot: Some(b64encode(snapshot.as_bytes())),
                update: None,
                version: b64encode(&doc.version_bytes()),
            }
        }
    };

    ws_tx
        .send(Message::Text(serde_json::to_string(&opened)?.into()))
        .await?;
    watched.push(remote_path.to_owned());
    Ok(())
}

async fn handle_update(
    state: &DriveSyncState,
    remote_path: &str,
    update_b64: &str,
    _base_version: Option<&str>,
    cmd_tx: &UnboundedSender<PeerCmd>,
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
            warn!("drive-sync p{peer_id} incomplete update for {remote_path}: {report:?}");
        }
        let _hash = state.save_doc(remote_path, &entry.doc).await?;

        let old_vv = std::mem::take(&mut entry.version);
        let export = entry.doc.export_updates_since_bytes(&old_vv);
        entry.version = entry.doc.version_bytes();
        if export.is_empty() {
            return Ok(());
        }
        (b64encode(&export), b64encode(&entry.version))
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
    state: &DriveSyncState,
    remote_path: &str,
    cmd_tx: &UnboundedSender<PeerCmd>,
) {
    state.unregister_peer(remote_path, cmd_tx).await;
    let has_peers = { state.peers.lock().await.contains_key(remote_path) };
    if !has_peers {
        let mut docs = state.docs.lock().await;
        docs.remove(remote_path);
        info!("drive-sync: evicted document cache for {remote_path}");
    }
}
