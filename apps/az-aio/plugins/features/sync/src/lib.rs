#![forbid(unsafe_code)]

automod::dir!(pub "src");

pub use descriptor::SyncPlugin;

pub use contracts::SyncWireMessage;
pub use finder_status::{FinderBadge, FinderHostedItem, FinderSyncState};
pub use sync_agent::{
    bootstrap_sync_agent, build_sync_agent_engine, default_roots_config_path,
    SyncAgentBootstrapReport, SyncAgentConfig, SyncAgentRoot, SyncAgentRootsConfig,
};
pub use sync_api::{sync_api_router, SyncApiMessage, SyncApiState};
pub use sync_client::{SyncWsConnection, SyncWsReader, SyncWsWriter};
pub use sync_engine::SyncEngine;
pub use sync_index::{
    default_local_index_path, SyncFileMetadata, SyncIndexRecord, SyncIndexSummary,
    SyncIndexedFileKind, SyncLocalIndex, SYNC_INDEX_SCHEMA_VERSION,
};
pub use sync_model::{
    SyncBlobKind, SyncCrdtEnvelope, SyncDeviceInfo, SyncDocumentRecord, SyncFileStatus, SyncRoot,
    DEFAULT_SYNC_ROOT_RELATIVE,
};
pub use sync_object_store::{
    sha256_file, FileSystemSyncObjectStore, RustfsSyncObjectStore, SyncFileSystemObjectStoreConfig,
    SyncObjectStoreConfig,
};
pub use sync_server::{
    InMemorySyncServerRepository, SyncObjectChunk, SyncObjectManifest, SyncPgRepository,
    SyncServerDeviceRecord, SyncServerFileRecord, SyncServerRootRecord, SyncServerSnapshot,
    SyncServerUpdateRecord, DEFAULT_OBJECT_CHUNK_BYTES, SYNC_SERVER_SCHEMA_SQL,
};
pub use sync_watcher::{
    SyncRenamePlan, SyncRootWatcher, SyncWatchEvent, SyncWatchEventKind, SyncWatchPlan,
    SyncWatchPlanner, DEFAULT_WATCH_DEBOUNCE_MS,
};
