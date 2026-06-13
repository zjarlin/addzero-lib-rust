#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]

automod::dir!(pub "src");

pub use descriptor::SyncPlugin;

#[cfg(not(target_arch = "wasm32"))]
pub use contracts::SyncWireMessage;
#[cfg(not(target_arch = "wasm32"))]
pub use finder_status::{FinderBadge, FinderHostedItem, FinderSyncState};
#[cfg(not(target_arch = "wasm32"))]
pub use sync_agent::{
    SyncAgentBootstrapReport, SyncAgentConfig, SyncAgentRoot, SyncAgentRootsConfig,
    bootstrap_sync_agent, build_sync_agent_engine, default_roots_config_path,
};
#[cfg(not(target_arch = "wasm32"))]
pub use sync_api::{SyncApiMessage, SyncApiState, sync_api_router};
#[cfg(not(target_arch = "wasm32"))]
pub use sync_client::{SyncWsConnection, SyncWsReader, SyncWsWriter};
#[cfg(not(target_arch = "wasm32"))]
pub use sync_engine::SyncEngine;
#[cfg(not(target_arch = "wasm32"))]
pub use sync_index::{
    SYNC_INDEX_SCHEMA_VERSION, SyncFileMetadata, SyncIndexRecord, SyncIndexSummary,
    SyncIndexedFileKind, SyncLocalIndex, default_local_index_path,
};
#[cfg(not(target_arch = "wasm32"))]
pub use sync_model::{
    DEFAULT_SYNC_ROOT_RELATIVE, SyncBlobKind, SyncCrdtEnvelope, SyncDeviceInfo, SyncDocumentRecord,
    SyncFileStatus, SyncRoot,
};
#[cfg(not(target_arch = "wasm32"))]
pub use sync_object_store::{
    FileSystemSyncObjectStore, RustfsSyncObjectStore, SyncFileSystemObjectStoreConfig,
    SyncObjectStoreConfig, sha256_file,
};
#[cfg(not(target_arch = "wasm32"))]
pub use sync_server::{
    DEFAULT_OBJECT_CHUNK_BYTES, InMemorySyncServerRepository, SYNC_SERVER_SCHEMA_SQL,
    SyncObjectChunk, SyncObjectManifest, SyncPgRepository, SyncServerDeviceRecord,
    SyncServerFileRecord, SyncServerRootRecord, SyncServerSnapshot, SyncServerUpdateRecord,
};
#[cfg(not(target_arch = "wasm32"))]
pub use sync_watcher::{
    DEFAULT_WATCH_DEBOUNCE_MS, SyncRenamePlan, SyncRootWatcher, SyncWatchEvent, SyncWatchEventKind,
    SyncWatchPlan, SyncWatchPlanner,
};
