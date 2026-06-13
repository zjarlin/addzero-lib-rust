#![cfg(not(target_arch = "wasm32"))]

use std::{
    collections::BTreeMap,
    sync::Arc,
    sync::atomic::{AtomicU64, Ordering},
};

use axum::{
    Json, Router,
    extract::ws::rejection::WebSocketUpgradeRejection,
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, broadcast};

use crate::{
    contracts::{
        SyncApplyTextRequest, SyncApplyTextResponse, SyncDeleteTextRequest, SyncFileListItem,
        SyncFilesQuery, SyncFilesResponse, SyncImportUpdateRequest, SyncImportUpdateResponse,
        SyncRootRequest, SyncStatusResponse, SyncWireMessage,
    },
    error::{SyncError, SyncResult},
    finder_status::FinderSyncState,
    sync_engine::SyncEngine,
    sync_model::{SyncCrdtEnvelope, SyncDeviceInfo, SyncDocumentRecord, SyncFileStatus, SyncRoot},
    sync_server::{
        SyncObjectManifest, SyncPgRepository, SyncServerFileRecord, SyncServerRootRecord,
        SyncServerUpdateRecord,
    },
};

#[derive(Clone)]
pub struct SyncApiState {
    engine: Arc<Mutex<SyncEngine>>,
    auth_token: Option<Arc<str>>,
    broadcasts: broadcast::Sender<SyncBroadcastEvent>,
    next_socket_id: Arc<AtomicU64>,
    connected_devices: Arc<Mutex<BTreeMap<String, SyncDeviceInfo>>>,
    object_manifests: Arc<Mutex<BTreeMap<(String, String), SyncKnownObjectManifest>>>,
    file_tombstones: Arc<Mutex<BTreeMap<String, SyncKnownFileTombstone>>>,
    repository: Option<SyncPgRepository>,
}

impl SyncApiState {
    pub fn new(engine: SyncEngine) -> Self {
        let (broadcasts, _) = broadcast::channel(128);
        Self {
            engine: Arc::new(Mutex::new(engine)),
            auth_token: None,
            broadcasts,
            next_socket_id: Arc::new(AtomicU64::new(1)),
            connected_devices: Arc::new(Mutex::new(BTreeMap::new())),
            object_manifests: Arc::new(Mutex::new(BTreeMap::new())),
            file_tombstones: Arc::new(Mutex::new(BTreeMap::new())),
            repository: None,
        }
    }

    pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {
        let token = token.into();
        if !token.trim().is_empty() {
            self.auth_token = Some(Arc::<str>::from(token));
        }
        self
    }

    pub fn with_pg_repository(mut self, repository: SyncPgRepository) -> Self {
        self.repository = Some(repository);
        self
    }

    pub async fn status(&self) -> SyncStatusResponse {
        let mut status = self.engine.lock().await.status();
        let connected_devices = self.connected_devices.lock().await;
        for device in connected_devices.values() {
            if !status
                .connected_devices
                .iter()
                .any(|existing| existing.device_name == device.device_name)
            {
                status.connected_devices.push(device.clone());
            }
        }
        status
    }

    pub async fn files(&self, query: SyncFilesQuery) -> SyncResult<SyncFilesResponse> {
        let space_id = query.space_id();
        let limit = query.normalized_limit();
        let cursor = query.normalized_cursor()?;
        if let Some(repository) = self.repository.as_ref() {
            return repository
                .list_files_page(&space_id, cursor.as_deref(), limit)
                .await;
        }
        let files = self
            .engine
            .lock()
            .await
            .files()
            .into_iter()
            .filter(|file| {
                cursor
                    .as_deref()
                    .map(|cursor| file.relative_path.as_str() > cursor)
                    .unwrap_or(true)
            })
            .take(limit.saturating_add(1))
            .map(|file| SyncFileListItem::from_document(space_id.clone(), &file))
            .collect::<Vec<_>>();
        Ok(files_response_from_items(space_id, files, limit))
    }

    pub async fn add_root(&self, request: SyncRootRequest) -> SyncResult<SyncStatusResponse> {
        let mut engine = self.engine.lock().await;
        let root = engine.add_root(
            request.alias,
            &request.relative_path,
            request.space_id.unwrap_or_else(|| "main".to_string()),
        )?;
        if let Some(repository) = self.repository.as_ref() {
            repository.register_device(engine.device()).await?;
            repository
                .upsert_root(&SyncServerRootRecord::from_root(
                    &engine.device().device_name,
                    &root,
                ))
                .await?;
        }
        Ok(engine.status())
    }

    pub async fn apply_text(
        &self,
        request: SyncApplyTextRequest,
    ) -> SyncResult<SyncApplyTextResponse> {
        let mut engine = self.engine.lock().await;
        let local_path = engine
            .device()
            .local_path_for_home_relative(&request.relative_path)?;
        let file = engine.apply_local_text(local_path, &request.text)?;
        let update = engine.export_update_since(&request.relative_path, None)?;
        if let Some(repository) = self.repository.as_ref() {
            repository.register_device(engine.device()).await?;
            repository
                .upsert_file(&SyncServerFileRecord::from_document("main", &file))
                .await?;
            repository
                .append_update(&SyncServerUpdateRecord::from_envelope(
                    "main",
                    update.clone(),
                )?)
                .await?;
        }
        Ok(SyncApplyTextResponse { file, update })
    }

    pub async fn delete_text(
        &self,
        request: SyncDeleteTextRequest,
    ) -> SyncResult<SyncApplyTextResponse> {
        let mut engine = self.engine.lock().await;
        let file = engine.delete_text(
            &request.relative_path,
            request.unicode_index,
            request.unicode_len,
        )?;
        let update = engine.export_update_since(&request.relative_path, None)?;
        if let Some(repository) = self.repository.as_ref() {
            repository.register_device(engine.device()).await?;
            repository
                .upsert_file(&SyncServerFileRecord::from_document("main", &file))
                .await?;
            repository
                .append_update(&SyncServerUpdateRecord::from_envelope(
                    "main",
                    update.clone(),
                )?)
                .await?;
        }
        Ok(SyncApplyTextResponse { file, update })
    }

    pub async fn import_update(
        &self,
        request: SyncImportUpdateRequest,
    ) -> SyncResult<SyncImportUpdateResponse> {
        let mut engine = self.engine.lock().await;
        let envelope = request.envelope;
        let file = engine.import_remote_blob(envelope.clone())?;
        if let Some(repository) = self.repository.as_ref() {
            repository.register_device(engine.device()).await?;
            repository
                .upsert_file(&SyncServerFileRecord::from_document("main", &file))
                .await?;
            repository
                .append_update(&SyncServerUpdateRecord::from_envelope("main", envelope)?)
                .await?;
        }
        Ok(SyncImportUpdateResponse {
            file,
            complete: true,
        })
    }

    pub async fn finder_state(&self) -> FinderSyncState {
        self.engine.lock().await.finder_state()
    }

    pub async fn refresh_finder_state(&self) -> SyncResult<FinderSyncState> {
        let engine = self.engine.lock().await;
        engine.write_default_finder_state()?;
        Ok(engine.finder_state())
    }
}

pub fn sync_api_router(state: SyncApiState) -> Router {
    Router::new()
        .route("/api/sync/status", get(status_handler))
        .route("/api/sync/files", get(files_handler))
        .route("/api/sync/roots", post(add_root_handler))
        .route("/api/sync/files/apply-text", post(apply_text_handler))
        .route("/api/sync/files/delete-text", post(delete_text_handler))
        .route("/api/sync/files/import-update", post(import_update_handler))
        .route("/api/sync/ws", get(ws_handler))
        .route("/api/sync/finder/status", get(finder_status_handler))
        .route("/api/sync/finder/refresh", post(finder_refresh_handler))
        .with_state(state)
}

async fn status_handler(State(state): State<SyncApiState>) -> Json<SyncStatusResponse> {
    Json(state.status().await)
}

async fn files_handler(
    State(state): State<SyncApiState>,
    Query(query): Query<SyncFilesQuery>,
) -> Result<Json<SyncFilesResponse>, SyncApiError> {
    state.files(query).await.map(Json).map_err(Into::into)
}

async fn add_root_handler(
    State(state): State<SyncApiState>,
    Json(request): Json<SyncRootRequest>,
) -> Result<Json<SyncStatusResponse>, SyncApiError> {
    state.add_root(request).await.map(Json).map_err(Into::into)
}

async fn apply_text_handler(
    State(state): State<SyncApiState>,
    Json(request): Json<SyncApplyTextRequest>,
) -> Result<Json<SyncApplyTextResponse>, SyncApiError> {
    state
        .apply_text(request)
        .await
        .map(Json)
        .map_err(Into::into)
}

async fn delete_text_handler(
    State(state): State<SyncApiState>,
    Json(request): Json<SyncDeleteTextRequest>,
) -> Result<Json<SyncApplyTextResponse>, SyncApiError> {
    state
        .delete_text(request)
        .await
        .map(Json)
        .map_err(Into::into)
}

async fn import_update_handler(
    State(state): State<SyncApiState>,
    Json(request): Json<SyncImportUpdateRequest>,
) -> Result<Json<SyncImportUpdateResponse>, SyncApiError> {
    state
        .import_update(request)
        .await
        .map(Json)
        .map_err(Into::into)
}

async fn ws_handler(
    State(state): State<SyncApiState>,
    headers: HeaderMap,
    Query(query): Query<SyncWsAuthQuery>,
    ws: Result<WebSocketUpgrade, WebSocketUpgradeRejection>,
) -> Result<Response, SyncApiError> {
    state.authorize_ws(&headers, query.token.as_deref())?;
    let ws = ws.map_err(SyncApiError::from_websocket_rejection)?;
    Ok(ws.on_upgrade(move |socket| run_sync_socket(state, socket)))
}

impl SyncApiState {
    fn authorize_ws(&self, headers: &HeaderMap, query_token: Option<&str>) -> SyncResult<()> {
        let Some(expected) = self.auth_token.as_deref() else {
            return Ok(());
        };
        let bearer_token = headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "));
        if bearer_token == Some(expected) || query_token == Some(expected) {
            return Ok(());
        }
        Err(SyncError::UnauthorizedWebSocket)
    }

    fn next_socket_id(&self) -> u64 {
        self.next_socket_id.fetch_add(1, Ordering::Relaxed)
    }

    fn subscribe(&self) -> broadcast::Receiver<SyncBroadcastEvent> {
        self.broadcasts.subscribe()
    }

    fn broadcast_from(&self, origin_socket_id: u64, message: SyncWireMessage) {
        let _ = self.broadcasts.send(SyncBroadcastEvent {
            origin_socket_id,
            message,
        });
    }

    async fn register_connected_device(&self, device: SyncDeviceInfo) -> SyncResult<()> {
        if let Some(repository) = self.repository.as_ref() {
            repository.register_device(&device).await?;
        }
        self.connected_devices
            .lock()
            .await
            .insert(device.device_name.clone(), device);
        Ok(())
    }

    async fn persist_roots(&self, device_name: &str, roots: &[SyncRoot]) -> SyncResult<()> {
        let Some(repository) = self.repository.as_ref() else {
            return Ok(());
        };
        for root in roots {
            repository
                .upsert_root(&SyncServerRootRecord::from_root(device_name, root))
                .await?;
        }
        Ok(())
    }

    async fn persist_file_update(
        &self,
        file: &SyncDocumentRecord,
        envelope: SyncCrdtEnvelope,
    ) -> SyncResult<()> {
        let Some(repository) = self.repository.as_ref() else {
            return Ok(());
        };
        repository
            .upsert_file(&SyncServerFileRecord::from_document("main", file))
            .await?;
        repository
            .append_update(&SyncServerUpdateRecord::from_envelope("main", envelope)?)
            .await?;
        Ok(())
    }

    async fn persist_object_manifest(
        &self,
        manifest: &SyncObjectManifest,
        source_device: &str,
    ) -> SyncResult<()> {
        self.object_manifests.lock().await.insert(
            (manifest.space_id.clone(), manifest.relative_path.clone()),
            SyncKnownObjectManifest {
                manifest: manifest.clone(),
                source_device: source_device.to_string(),
            },
        );
        if let Some(repository) = self.repository.as_ref() {
            repository.upsert_object_manifest(manifest).await?;
        }
        Ok(())
    }

    async fn persist_file_tombstone(
        &self,
        relative_path: &str,
        source_device: &str,
    ) -> SyncResult<String> {
        let relative_path = crate::sync_model::normalize_home_relative_path(relative_path)?;
        self.file_tombstones.lock().await.insert(
            relative_path.clone(),
            SyncKnownFileTombstone {
                relative_path: relative_path.clone(),
                source_device: source_device.to_string(),
            },
        );
        self.object_manifests
            .lock()
            .await
            .retain(|(_space_id, object_path), _known| object_path != &relative_path);
        if let Some(repository) = self.repository.as_ref() {
            repository
                .upsert_file(&SyncServerFileRecord {
                    space_id: "main".to_string(),
                    relative_path: relative_path.clone(),
                    file_kind: crate::sync_index::SyncIndexedFileKind::Missing,
                    content_hash: String::new(),
                    crdt_version: Vec::new(),
                    status: SyncFileStatus::Deleted,
                    size_bytes: None,
                    updated_by_device: source_device.to_string(),
                })
                .await?;
        }
        Ok(relative_path)
    }
}

async fn run_sync_socket(state: SyncApiState, mut socket: WebSocket) {
    let socket_id = state.next_socket_id();
    let mut broadcasts = state.subscribe();
    loop {
        tokio::select! {
            message = socket.recv() => {
                let Some(message) = message else {
                    break;
                };
                let Ok(message) = message else {
                    break;
                };
                let responses = match message {
                    Message::Text(text) => match serde_json::from_str::<SyncWireMessage>(&text) {
                        Ok(message) => handle_wire_message(&state, socket_id, message).await,
                        Err(error) => vec![SyncWireMessage::Error {
                            message: format!("invalid sync message: {error}"),
                        }],
                    },
                    Message::Close(_) => break,
                    Message::Ping(payload) => {
                        let _ = socket.send(Message::Pong(payload)).await;
                        continue;
                    }
                    Message::Pong(_) => continue,
                    Message::Binary(_) => vec![SyncWireMessage::Error {
                        message: "binary WebSocket frames are not part of the sync protocol yet"
                            .to_string(),
                    }],
                };
                let mut send_failed = false;
                for response in responses {
                    if send_wire_message(&mut socket, &response).await.is_err() {
                        send_failed = true;
                        break;
                    }
                }
                if send_failed {
                    break;
                }
            }
            event = broadcasts.recv() => {
                match event {
                    Ok(event) if event.origin_socket_id != socket_id => {
                        if send_wire_message(&mut socket, &event.message).await.is_err() {
                            break;
                        }
                    }
                    Ok(_) | Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
}

async fn send_wire_message(socket: &mut WebSocket, message: &SyncWireMessage) -> SyncResult<()> {
    let text = serde_json::to_string(message).map_err(SyncError::WireJson)?;
    socket
        .send(Message::Text(text.into()))
        .await
        .map_err(|error| SyncError::WebSocketUpgrade(error.to_string()))
}

async fn handle_wire_message(
    state: &SyncApiState,
    socket_id: u64,
    message: SyncWireMessage,
) -> Vec<SyncWireMessage> {
    match message {
        SyncWireMessage::Hello { device, roots } => {
            let device_name = device.device_name.clone();
            if let Err(error) = state.register_connected_device(device).await {
                return vec![SyncWireMessage::Error {
                    message: error.to_string(),
                }];
            }
            if let Err(error) = state.persist_roots(&device_name, &roots).await {
                return vec![SyncWireMessage::Error {
                    message: error.to_string(),
                }];
            }
            let mut responses = vec![SyncWireMessage::Heartbeat { device_name }];
            let engine = state.engine.lock().await;
            for file in engine.files() {
                match engine.export_update_since(&file.relative_path, None) {
                    Ok(envelope) => responses.push(SyncWireMessage::Update { envelope }),
                    Err(error) => responses.push(SyncWireMessage::Error {
                        message: error.to_string(),
                    }),
                }
            }
            drop(engine);
            let object_manifests = state.object_manifests.lock().await;
            for known in object_manifests.values() {
                responses.push(SyncWireMessage::ObjectManifest {
                    manifest: known.manifest.clone(),
                    source_device: known.source_device.clone(),
                });
            }
            drop(object_manifests);
            let file_tombstones = state.file_tombstones.lock().await;
            for tombstone in file_tombstones.values() {
                responses.push(SyncWireMessage::FileDeleted {
                    relative_path: tombstone.relative_path.clone(),
                    source_device: tombstone.source_device.clone(),
                });
            }
            responses
        }
        SyncWireMessage::Heartbeat { device_name } => {
            vec![SyncWireMessage::Heartbeat { device_name }]
        }
        SyncWireMessage::Update { envelope } => {
            let broadcast_envelope = envelope.clone();
            let mut engine = state.engine.lock().await;
            match engine.import_remote_blob(envelope) {
                Ok(file) => {
                    if let Err(error) = state
                        .persist_file_update(&file, broadcast_envelope.clone())
                        .await
                    {
                        return vec![SyncWireMessage::Error {
                            message: error.to_string(),
                        }];
                    }
                    state.broadcast_from(
                        socket_id,
                        SyncWireMessage::Update {
                            envelope: broadcast_envelope,
                        },
                    );
                    vec![SyncWireMessage::Ack {
                        relative_path: file.relative_path,
                        version: file.crdt_version,
                    }]
                }
                Err(error) => vec![SyncWireMessage::Error {
                    message: error.to_string(),
                }],
            }
        }
        SyncWireMessage::RequestSnapshot { relative_path } => {
            let engine = state.engine.lock().await;
            match engine.export_snapshot(&relative_path) {
                Ok(envelope) => vec![SyncWireMessage::Snapshot { envelope }],
                Err(error) => vec![SyncWireMessage::Error {
                    message: error.to_string(),
                }],
            }
        }
        SyncWireMessage::Ack {
            relative_path,
            version,
        } => vec![SyncWireMessage::Ack {
            relative_path,
            version,
        }],
        SyncWireMessage::Snapshot { envelope } => {
            let broadcast_envelope = envelope.clone();
            let mut engine = state.engine.lock().await;
            match engine.import_remote_blob(envelope) {
                Ok(file) => {
                    if let Err(error) = state
                        .persist_file_update(&file, broadcast_envelope.clone())
                        .await
                    {
                        return vec![SyncWireMessage::Error {
                            message: error.to_string(),
                        }];
                    }
                    state.broadcast_from(
                        socket_id,
                        SyncWireMessage::Snapshot {
                            envelope: broadcast_envelope,
                        },
                    );
                    vec![SyncWireMessage::Ack {
                        relative_path: file.relative_path,
                        version: file.crdt_version,
                    }]
                }
                Err(error) => vec![SyncWireMessage::Error {
                    message: error.to_string(),
                }],
            }
        }
        SyncWireMessage::ObjectManifest {
            manifest,
            source_device,
        } => {
            if let Err(error) = state
                .persist_object_manifest(&manifest, &source_device)
                .await
            {
                return vec![SyncWireMessage::Error {
                    message: error.to_string(),
                }];
            }
            state.broadcast_from(
                socket_id,
                SyncWireMessage::ObjectManifest {
                    manifest: manifest.clone(),
                    source_device: source_device.clone(),
                },
            );
            vec![SyncWireMessage::Ack {
                relative_path: manifest.relative_path,
                version: source_device.into_bytes(),
            }]
        }
        SyncWireMessage::FileDeleted {
            relative_path,
            source_device,
        } => match state
            .persist_file_tombstone(&relative_path, &source_device)
            .await
        {
            Ok(relative_path) => {
                state.broadcast_from(
                    socket_id,
                    SyncWireMessage::FileDeleted {
                        relative_path: relative_path.clone(),
                        source_device: source_device.clone(),
                    },
                );
                vec![SyncWireMessage::Ack {
                    relative_path,
                    version: source_device.into_bytes(),
                }]
            }
            Err(error) => vec![SyncWireMessage::Error {
                message: error.to_string(),
            }],
        },
        SyncWireMessage::Error { message } => vec![SyncWireMessage::Error { message }],
    }
}

async fn finder_status_handler(State(state): State<SyncApiState>) -> Json<FinderSyncState> {
    Json(state.finder_state().await)
}

async fn finder_refresh_handler(
    State(state): State<SyncApiState>,
) -> Result<Json<FinderSyncState>, SyncApiError> {
    state
        .refresh_finder_state()
        .await
        .map(Json)
        .map_err(Into::into)
}

#[derive(Debug)]
struct SyncApiError(SyncError);

impl From<SyncError> for SyncApiError {
    fn from(value: SyncError) -> Self {
        Self(value)
    }
}

impl SyncApiError {
    fn from_websocket_rejection(value: WebSocketUpgradeRejection) -> Self {
        Self(SyncError::WebSocketUpgrade(value.to_string()))
    }
}

impl IntoResponse for SyncApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            SyncError::InvalidRelativePath { .. }
            | SyncError::InvalidFileKind { .. }
            | SyncError::InvalidFileStatus { .. }
            | SyncError::PathOutsideHome { .. }
            | SyncError::IndexInsideSyncRoot { .. } => StatusCode::BAD_REQUEST,
            SyncError::UnauthorizedWebSocket => StatusCode::UNAUTHORIZED,
            SyncError::WebSocketUpgrade(_) => StatusCode::UPGRADE_REQUIRED,
            SyncError::MissingDocument { .. } => StatusCode::NOT_FOUND,
            SyncError::Crdt { .. }
            | SyncError::Io { .. }
            | SyncError::Json { .. }
            | SyncError::ObjectHashMismatch { .. }
            | SyncError::ObjectStorage(_)
            | SyncError::Watch { .. }
            | SyncError::WebSocketTransport(_)
            | SyncError::WebSocketAuthHeader(_)
            | SyncError::WireJson(_)
            | SyncError::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (
            status,
            Json(SyncApiErrorBody {
                error: self.0.to_string(),
            }),
        )
            .into_response()
    }
}

fn files_response_from_items(
    space_id: String,
    mut files: Vec<SyncFileListItem>,
    limit: usize,
) -> SyncFilesResponse {
    let next_cursor = if files.len() > limit {
        files.pop();
        files.last().map(|file| file.relative_path.clone())
    } else {
        None
    };
    SyncFilesResponse {
        space_id,
        files,
        next_cursor,
        limit,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncApiMessage {
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct SyncApiErrorBody {
    error: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize)]
struct SyncWsAuthQuery {
    token: Option<String>,
}

#[derive(Clone, Debug)]
struct SyncBroadcastEvent {
    origin_socket_id: u64,
    message: SyncWireMessage,
}

#[derive(Clone, Debug)]
struct SyncKnownObjectManifest {
    manifest: SyncObjectManifest,
    source_device: String,
}

#[derive(Clone, Debug)]
struct SyncKnownFileTombstone {
    relative_path: String,
    source_device: String,
}

#[cfg(test)]
mod tests {
    use std::{fs, time::Duration};

    use axum::{
        body::Body,
        http::{Method, Request, StatusCode, header},
    };
    use az_line_crdt::LineCrdtVersion;
    use http_body_util::BodyExt;
    use tempfile::TempDir;
    use tokio::{net::TcpListener, task::JoinHandle, time::timeout};
    use tower::ServiceExt;

    use crate::{
        contracts::{
            SyncApplyTextResponse, SyncFilesResponse, SyncStatusResponse, SyncWireMessage,
        },
        sync_api::{SyncApiState, sync_api_router},
        sync_client::{SyncWsConnection, SyncWsReader},
        sync_engine::SyncEngine,
        sync_model::SyncDeviceInfo,
        sync_object_store::{FileSystemSyncObjectStore, SyncFileSystemObjectStoreConfig},
        sync_server::SyncObjectManifest,
    };

    #[tokio::test]
    async fn status_route_exposes_default_root_and_index() -> Result<(), Box<dyn std::error::Error>>
    {
        let app = test_router("/tmp/sync-api-a");
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/sync/status")
                    .body(Body::empty())?,
            )
            .await?;
        let status: SyncStatusResponse = response_json(response).await?;

        assert_eq!(status.roots[0].relative_path, "az-sync");
        assert!(status.local_index.stored_outside_sync_roots);
        Ok(())
    }

    #[tokio::test]
    async fn apply_and_delete_text_routes_emit_crdt_updates()
    -> Result<(), Box<dyn std::error::Error>> {
        let app = test_router("/tmp/sync-api-b");
        let response = app
            .oneshot(json_request(
                "/api/sync/files/apply-text",
                r#"{"relative_path":"az-sync/a.txt","text":"one\ntwo\nthree"}"#,
            )?)
            .await?;
        let applied: SyncApplyTextResponse = response_json(response).await?;
        assert!(!applied.update.blob.is_empty());

        let app = test_router_with_text("/tmp/sync-api-c", "one\ntwo\nthree")?;
        let response = app
            .oneshot(json_request(
                "/api/sync/files/delete-text",
                r#"{"relative_path":"az-sync/a.txt","unicode_index":4,"unicode_len":3}"#,
            )?)
            .await?;
        let deleted: SyncApplyTextResponse = response_json(response).await?;
        assert_eq!(deleted.file.content_hash, deleted.update.content_hash);
        Ok(())
    }

    #[tokio::test]
    async fn files_route_pages_by_home_relative_cursor() -> Result<(), Box<dyn std::error::Error>> {
        let app = test_router_with_files(
            "/tmp/sync-api-files",
            &[
                ("az-sync/a.txt", "alpha"),
                ("az-sync/b.txt", "bravo"),
                ("az-sync/c.txt", "charlie"),
            ],
        )?;
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/sync/files?limit=1")
                    .body(Body::empty())?,
            )
            .await?;
        let page: SyncFilesResponse = response_json(response).await?;

        assert_eq!(
            page.files
                .iter()
                .map(|file| file.relative_path.as_str())
                .collect::<Vec<_>>(),
            vec!["az-sync/a.txt"]
        );
        assert_eq!(page.next_cursor.as_deref(), Some("az-sync/a.txt"));

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/sync/files?cursor=az-sync/a.txt&limit=2")
                    .body(Body::empty())?,
            )
            .await?;
        let page: SyncFilesResponse = response_json(response).await?;
        assert_eq!(
            page.files
                .iter()
                .map(|file| file.relative_path.as_str())
                .collect::<Vec<_>>(),
            vec!["az-sync/b.txt", "az-sync/c.txt"]
        );
        assert_eq!(page.next_cursor, None);
        Ok(())
    }

    #[tokio::test]
    async fn files_route_rejects_home_escape_cursor() -> Result<(), Box<dyn std::error::Error>> {
        let app = test_router("/tmp/sync-api-files-bad-cursor");
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/sync/files?cursor=../secret")
                    .body(Body::empty())?,
            )
            .await?;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        Ok(())
    }

    #[tokio::test]
    async fn invalid_root_path_returns_bad_request() -> Result<(), Box<dyn std::error::Error>> {
        let app = test_router("/tmp/sync-api-d");
        let response = app
            .oneshot(json_request(
                "/api/sync/roots",
                r#"{"alias":"bad","relative_path":"../secret","space_id":"main"}"#,
            )?)
            .await?;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        Ok(())
    }

    #[tokio::test]
    async fn websocket_route_requires_configured_token() -> Result<(), Box<dyn std::error::Error>> {
        let app = test_router_with_token("/tmp/sync-api-e", "secret");
        let response = app.oneshot(websocket_request(None)?).await?;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        Ok(())
    }

    #[tokio::test]
    async fn websocket_route_accepts_authenticated_upgrade_request()
    -> Result<(), Box<dyn std::error::Error>> {
        let app = test_router_with_token("/tmp/sync-api-f", "secret");
        let response = app
            .oneshot(websocket_request(Some("Bearer secret"))?)
            .await?;

        assert_eq!(response.status(), StatusCode::UPGRADE_REQUIRED);
        Ok(())
    }

    #[tokio::test]
    async fn websocket_two_device_flow_converges_after_remote_delete()
    -> Result<(), Box<dyn std::error::Error>> {
        let server_home = tempfile::tempdir()?;
        let a_home = tempfile::tempdir()?;
        let b_home = tempfile::tempdir()?;
        let (endpoint, state, server_task) = spawn_websocket_server(&server_home).await?;
        let mut left = SyncEngine::with_device(SyncDeviceInfo::new("agent-a", a_home.path()));
        let mut right = SyncEngine::with_device(SyncDeviceInfo::new("agent-b", b_home.path()));
        let SyncWsConnection {
            writer: mut left_writer,
            reader: mut left_reader,
        } = SyncWsConnection::connect(&endpoint, None).await?;
        let SyncWsConnection {
            writer: mut right_writer,
            reader: mut right_reader,
        } = SyncWsConnection::connect(&endpoint, None).await?;

        left_writer
            .send(&SyncWireMessage::Hello {
                device: left.device().clone(),
                roots: left.roots(),
            })
            .await?;
        assert_heartbeat(next_ws_message(&mut left_reader).await?, "agent-a");
        right_writer
            .send(&SyncWireMessage::Hello {
                device: right.device().clone(),
                roots: right.roots(),
            })
            .await?;
        assert_heartbeat(next_ws_message(&mut right_reader).await?, "agent-b");

        let status = state.status().await;
        assert_connected_devices(&status, &["server-a", "agent-a", "agent-b"]);

        let left_record =
            left.apply_local_text(a_home.path().join("az-sync/a.txt"), "one\ntwo\nthree")?;
        left_writer
            .send(&SyncWireMessage::Update {
                envelope: left.export_update_since("az-sync/a.txt", None)?,
            })
            .await?;
        assert_ack(next_ws_message(&mut left_reader).await?, "az-sync/a.txt");
        let first_remote = expect_update(next_ws_message(&mut right_reader).await?);
        right.import_remote_blob(first_remote)?;
        let right_path = right.materialize_text_to_local_file("az-sync/a.txt")?;
        assert_eq!(fs::read_to_string(right_path)?, "one\ntwo\nthree");

        right.delete_text("az-sync/a.txt", 4, 3)?;
        let left_version = LineCrdtVersion::from_bytes(left_record.crdt_version);
        right_writer
            .send(&SyncWireMessage::Update {
                envelope: right.export_update_since("az-sync/a.txt", Some(&left_version))?,
            })
            .await?;
        assert_ack(next_ws_message(&mut right_reader).await?, "az-sync/a.txt");
        let second_remote = expect_update(next_ws_message(&mut left_reader).await?);
        left.import_remote_blob(second_remote)?;
        let left_path = left.materialize_text_to_local_file("az-sync/a.txt")?;

        assert_eq!(fs::read_to_string(left_path)?, "one\n\nthree");
        assert_eq!(
            left.materialize_text("az-sync/a.txt")?,
            right.materialize_text("az-sync/a.txt")?
        );
        server_task.abort();
        Ok(())
    }

    #[tokio::test]
    async fn websocket_hello_replays_server_known_text_updates_to_new_device()
    -> Result<(), Box<dyn std::error::Error>> {
        let server_home = tempfile::tempdir()?;
        let a_home = tempfile::tempdir()?;
        let b_home = tempfile::tempdir()?;
        let (endpoint, _state, server_task) = spawn_websocket_server(&server_home).await?;
        let mut left = SyncEngine::with_device(SyncDeviceInfo::new("agent-a", a_home.path()));
        let mut right = SyncEngine::with_device(SyncDeviceInfo::new("agent-b", b_home.path()));
        left.apply_local_text(a_home.path().join("az-sync/existing.txt"), "alpha\nbeta")?;
        let SyncWsConnection {
            writer: mut left_writer,
            reader: mut left_reader,
        } = SyncWsConnection::connect(&endpoint, None).await?;

        left_writer
            .send(&SyncWireMessage::Hello {
                device: left.device().clone(),
                roots: left.roots(),
            })
            .await?;
        assert_heartbeat(next_ws_message(&mut left_reader).await?, "agent-a");
        left_writer
            .send(&SyncWireMessage::Update {
                envelope: left.export_update_since("az-sync/existing.txt", None)?,
            })
            .await?;
        assert_ack(
            next_ws_message(&mut left_reader).await?,
            "az-sync/existing.txt",
        );

        let SyncWsConnection {
            writer: mut right_writer,
            reader: mut right_reader,
        } = SyncWsConnection::connect(&endpoint, None).await?;
        right_writer
            .send(&SyncWireMessage::Hello {
                device: right.device().clone(),
                roots: right.roots(),
            })
            .await?;
        assert_heartbeat(next_ws_message(&mut right_reader).await?, "agent-b");
        let replayed = expect_update(next_ws_message(&mut right_reader).await?);
        right.import_remote_blob(replayed)?;
        let right_path = right.materialize_text_to_local_file("az-sync/existing.txt")?;

        assert_eq!(fs::read_to_string(right_path)?, "alpha\nbeta");
        server_task.abort();
        Ok(())
    }

    #[tokio::test]
    async fn websocket_file_deleted_broadcasts_and_replays_to_new_device()
    -> Result<(), Box<dyn std::error::Error>> {
        let server_home = tempfile::tempdir()?;
        let a_home = tempfile::tempdir()?;
        let b_home = tempfile::tempdir()?;
        let c_home = tempfile::tempdir()?;
        let (endpoint, _state, server_task) = spawn_websocket_server(&server_home).await?;
        let left = SyncEngine::with_device(SyncDeviceInfo::new("agent-a", a_home.path()));
        let right = SyncEngine::with_device(SyncDeviceInfo::new("agent-b", b_home.path()));
        let late = SyncEngine::with_device(SyncDeviceInfo::new("agent-c", c_home.path()));
        let SyncWsConnection {
            writer: mut left_writer,
            reader: mut left_reader,
        } = SyncWsConnection::connect(&endpoint, None).await?;
        let SyncWsConnection {
            writer: mut right_writer,
            reader: mut right_reader,
        } = SyncWsConnection::connect(&endpoint, None).await?;

        left_writer
            .send(&SyncWireMessage::Hello {
                device: left.device().clone(),
                roots: left.roots(),
            })
            .await?;
        assert_heartbeat(next_ws_message(&mut left_reader).await?, "agent-a");
        right_writer
            .send(&SyncWireMessage::Hello {
                device: right.device().clone(),
                roots: right.roots(),
            })
            .await?;
        assert_heartbeat(next_ws_message(&mut right_reader).await?, "agent-b");

        left_writer
            .send(&SyncWireMessage::FileDeleted {
                relative_path: "az-sync/deleted.txt".to_string(),
                source_device: left.device().device_name.clone(),
            })
            .await?;
        assert_ack(
            next_ws_message(&mut left_reader).await?,
            "az-sync/deleted.txt",
        );
        assert_file_deleted(
            next_ws_message(&mut right_reader).await?,
            "az-sync/deleted.txt",
            "agent-a",
        );

        let SyncWsConnection {
            writer: mut late_writer,
            reader: mut late_reader,
        } = SyncWsConnection::connect(&endpoint, None).await?;
        late_writer
            .send(&SyncWireMessage::Hello {
                device: late.device().clone(),
                roots: late.roots(),
            })
            .await?;
        assert_heartbeat(next_ws_message(&mut late_reader).await?, "agent-c");
        assert_file_deleted(
            next_ws_message(&mut late_reader).await?,
            "az-sync/deleted.txt",
            "agent-a",
        );
        server_task.abort();
        Ok(())
    }

    #[tokio::test]
    async fn websocket_object_manifest_flow_restores_binary_file()
    -> Result<(), Box<dyn std::error::Error>> {
        let server_home = tempfile::tempdir()?;
        let a_home = tempfile::tempdir()?;
        let b_home = tempfile::tempdir()?;
        let object_root = tempfile::tempdir()?;
        let (endpoint, _state, server_task) = spawn_websocket_server(&server_home).await?;
        let left = SyncEngine::with_device(SyncDeviceInfo::new("agent-a", a_home.path()));
        let right = SyncEngine::with_device(SyncDeviceInfo::new("agent-b", b_home.path()));
        let object_store = FileSystemSyncObjectStore::new(
            SyncFileSystemObjectStoreConfig::new(object_root.path()).with_chunk_size_bytes(3),
        );
        let SyncWsConnection {
            writer: mut left_writer,
            reader: mut left_reader,
        } = SyncWsConnection::connect(&endpoint, None).await?;
        let SyncWsConnection {
            writer: mut right_writer,
            reader: mut right_reader,
        } = SyncWsConnection::connect(&endpoint, None).await?;

        left_writer
            .send(&SyncWireMessage::Hello {
                device: left.device().clone(),
                roots: left.roots(),
            })
            .await?;
        assert_heartbeat(next_ws_message(&mut left_reader).await?, "agent-a");
        right_writer
            .send(&SyncWireMessage::Hello {
                device: right.device().clone(),
                roots: right.roots(),
            })
            .await?;
        assert_heartbeat(next_ws_message(&mut right_reader).await?, "agent-b");

        let source_path = a_home.path().join("az-sync/blob.bin");
        fs::create_dir_all(source_path.parent().expect("test binary parent"))?;
        let bytes = vec![0, 159, 146, 150, 8, 13, 21, 34, 55];
        fs::write(&source_path, &bytes)?;
        let manifest = object_store.put_file("main", "az-sync/blob.bin", &source_path)?;
        left_writer
            .send(&SyncWireMessage::ObjectManifest {
                manifest: manifest.clone(),
                source_device: left.device().device_name.clone(),
            })
            .await?;
        assert_ack(next_ws_message(&mut left_reader).await?, "az-sync/blob.bin");

        let remote_manifest = expect_object_manifest(next_ws_message(&mut right_reader).await?);
        let target_path = right
            .device()
            .local_path_for_home_relative(&remote_manifest.relative_path)?;
        object_store.materialize_file(&remote_manifest, &target_path)?;

        assert_eq!(remote_manifest, manifest);
        assert_eq!(fs::read(target_path)?, bytes);
        server_task.abort();
        Ok(())
    }

    #[tokio::test]
    async fn websocket_hello_replays_known_object_manifest_to_new_device()
    -> Result<(), Box<dyn std::error::Error>> {
        let server_home = tempfile::tempdir()?;
        let a_home = tempfile::tempdir()?;
        let b_home = tempfile::tempdir()?;
        let object_root = tempfile::tempdir()?;
        let (endpoint, _state, server_task) = spawn_websocket_server(&server_home).await?;
        let left = SyncEngine::with_device(SyncDeviceInfo::new("agent-a", a_home.path()));
        let right = SyncEngine::with_device(SyncDeviceInfo::new("agent-b", b_home.path()));
        let object_store = FileSystemSyncObjectStore::new(
            SyncFileSystemObjectStoreConfig::new(object_root.path()).with_chunk_size_bytes(3),
        );
        let SyncWsConnection {
            writer: mut left_writer,
            reader: mut left_reader,
        } = SyncWsConnection::connect(&endpoint, None).await?;

        left_writer
            .send(&SyncWireMessage::Hello {
                device: left.device().clone(),
                roots: left.roots(),
            })
            .await?;
        assert_heartbeat(next_ws_message(&mut left_reader).await?, "agent-a");
        let source_path = a_home.path().join("az-sync/preexisting.bin");
        fs::create_dir_all(source_path.parent().expect("test binary parent"))?;
        let bytes = vec![0, 159, 146, 150, 8, 13, 21, 34, 55];
        fs::write(&source_path, &bytes)?;
        let manifest = object_store.put_file("main", "az-sync/preexisting.bin", &source_path)?;
        left_writer
            .send(&SyncWireMessage::ObjectManifest {
                manifest: manifest.clone(),
                source_device: left.device().device_name.clone(),
            })
            .await?;
        assert_ack(
            next_ws_message(&mut left_reader).await?,
            "az-sync/preexisting.bin",
        );

        let SyncWsConnection {
            writer: mut right_writer,
            reader: mut right_reader,
        } = SyncWsConnection::connect(&endpoint, None).await?;
        right_writer
            .send(&SyncWireMessage::Hello {
                device: right.device().clone(),
                roots: right.roots(),
            })
            .await?;
        assert_heartbeat(next_ws_message(&mut right_reader).await?, "agent-b");
        let remote_manifest = expect_object_manifest(next_ws_message(&mut right_reader).await?);
        let target_path = right
            .device()
            .local_path_for_home_relative(&remote_manifest.relative_path)?;
        object_store.materialize_file(&remote_manifest, &target_path)?;

        assert_eq!(remote_manifest, manifest);
        assert_eq!(fs::read(target_path)?, bytes);
        server_task.abort();
        Ok(())
    }

    fn test_router(home: &str) -> axum::Router {
        let engine = SyncEngine::with_device(SyncDeviceInfo::new("api-test", home));
        sync_api_router(SyncApiState::new(engine))
    }

    fn test_router_with_token(home: &str, token: &str) -> axum::Router {
        let engine = SyncEngine::with_device(SyncDeviceInfo::new("api-test", home));
        sync_api_router(SyncApiState::new(engine).with_auth_token(token))
    }

    fn test_router_with_text(
        home: &str,
        text: &str,
    ) -> Result<axum::Router, Box<dyn std::error::Error>> {
        let mut engine = SyncEngine::with_device(SyncDeviceInfo::new("api-test", home));
        engine.apply_local_text(format!("{home}/az-sync/a.txt"), text)?;
        Ok(sync_api_router(SyncApiState::new(engine)))
    }

    fn test_router_with_files(
        home: &str,
        files: &[(&str, &str)],
    ) -> Result<axum::Router, Box<dyn std::error::Error>> {
        let mut engine = SyncEngine::with_device(SyncDeviceInfo::new("api-test", home));
        for (relative_path, text) in files {
            engine.apply_local_text(format!("{home}/{relative_path}"), text)?;
        }
        Ok(sync_api_router(SyncApiState::new(engine)))
    }

    async fn spawn_websocket_server(
        server_home: &TempDir,
    ) -> Result<(String, SyncApiState, JoinHandle<()>), Box<dyn std::error::Error>> {
        let state = SyncApiState::new(SyncEngine::with_device(SyncDeviceInfo::new(
            "server-a",
            server_home.path(),
        )));
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let app = sync_api_router(state.clone());
        let task = tokio::spawn(async move {
            if let Err(error) = axum::serve(listener, app).await {
                eprintln!("sync websocket test server failed: {error}");
            }
        });
        Ok((format!("ws://{address}/api/sync/ws"), state, task))
    }

    fn json_request(
        uri: &str,
        body: &'static str,
    ) -> Result<Request<Body>, Box<dyn std::error::Error>> {
        Ok(Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))?)
    }

    fn websocket_request(
        authorization: Option<&str>,
    ) -> Result<Request<Body>, Box<dyn std::error::Error>> {
        let mut builder = Request::builder()
            .method(Method::GET)
            .uri("/api/sync/ws")
            .header(header::CONNECTION, "upgrade")
            .header(header::UPGRADE, "websocket")
            .header(header::SEC_WEBSOCKET_VERSION, "13")
            .header(header::SEC_WEBSOCKET_KEY, "dGhlIHNhbXBsZSBub25jZQ==");
        if let Some(authorization) = authorization {
            builder = builder.header(header::AUTHORIZATION, authorization);
        }
        Ok(builder.body(Body::empty())?)
    }

    async fn next_ws_message(
        reader: &mut SyncWsReader,
    ) -> Result<SyncWireMessage, Box<dyn std::error::Error>> {
        timeout(Duration::from_secs(5), reader.recv())
            .await??
            .ok_or_else(|| "sync WebSocket closed before next message".into())
    }

    fn assert_heartbeat(message: SyncWireMessage, expected_device_name: &str) {
        match message {
            SyncWireMessage::Heartbeat { device_name } => {
                assert_eq!(device_name, expected_device_name);
            }
            other => panic!("expected heartbeat, got {other:?}"),
        }
    }

    fn assert_ack(message: SyncWireMessage, expected_relative_path: &str) {
        match message {
            SyncWireMessage::Ack { relative_path, .. } => {
                assert_eq!(relative_path, expected_relative_path);
            }
            other => panic!("expected ack, got {other:?}"),
        }
    }

    fn expect_update(message: SyncWireMessage) -> crate::sync_model::SyncCrdtEnvelope {
        match message {
            SyncWireMessage::Update { envelope } => envelope,
            other => panic!("expected update, got {other:?}"),
        }
    }

    fn expect_object_manifest(message: SyncWireMessage) -> SyncObjectManifest {
        match message {
            SyncWireMessage::ObjectManifest { manifest, .. } => manifest,
            other => panic!("expected object manifest, got {other:?}"),
        }
    }

    fn assert_file_deleted(
        message: SyncWireMessage,
        expected_relative_path: &str,
        expected_source_device: &str,
    ) {
        match message {
            SyncWireMessage::FileDeleted {
                relative_path,
                source_device,
            } => {
                assert_eq!(relative_path, expected_relative_path);
                assert_eq!(source_device, expected_source_device);
            }
            other => panic!("expected file deleted, got {other:?}"),
        }
    }

    fn assert_connected_devices(status: &SyncStatusResponse, expected_names: &[&str]) {
        for expected_name in expected_names {
            assert!(
                status
                    .connected_devices
                    .iter()
                    .any(|device| device.device_name == *expected_name),
                "expected connected device `{expected_name}` in {:?}",
                status.connected_devices
            );
        }
    }

    async fn response_json<T: serde::de::DeserializeOwned>(
        response: axum::response::Response,
    ) -> Result<T, Box<dyn std::error::Error>> {
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await?.to_bytes();
        Ok(serde_json::from_slice(&bytes)?)
    }
}
