#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]

mod descriptor;

#[cfg(not(target_arch = "wasm32"))]
pub mod contracts;
#[cfg(not(target_arch = "wasm32"))]
pub mod error;
#[cfg(not(target_arch = "wasm32"))]
pub mod finder_status;
#[cfg(not(target_arch = "wasm32"))]
pub mod sync_agent;
#[cfg(not(target_arch = "wasm32"))]
pub mod sync_api;
#[cfg(not(target_arch = "wasm32"))]
pub mod sync_client;
#[cfg(not(target_arch = "wasm32"))]
pub mod sync_engine;
#[cfg(not(target_arch = "wasm32"))]
pub mod sync_index;
#[cfg(not(target_arch = "wasm32"))]
pub mod sync_model;
#[cfg(not(target_arch = "wasm32"))]
pub mod sync_object_store;
#[cfg(not(target_arch = "wasm32"))]
pub mod sync_server;
#[cfg(not(target_arch = "wasm32"))]
pub mod sync_watcher;

pub use descriptor::SyncPlugin;

#[cfg(not(target_arch = "wasm32"))]
pub use contracts::SyncWireMessage;
#[cfg(not(target_arch = "wasm32"))]
pub use error::{SyncError, SyncResult};
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

#[cfg(target_arch = "wasm32")]
mod component {
    use az_aio_plugin_api::{AzAioPlugin, PluginKind, contributions_to_json, descriptor_to_json};

    use super::SyncPlugin;

    wit_bindgen::generate!({
        path: "../../wit",
        world: "az-aio-plugin",
    });

    struct SyncWasm;

    impl Guest for SyncWasm {
        fn describe() -> Result<String, String> {
            let mut descriptor = SyncPlugin.descriptor();
            descriptor.kind = PluginKind::WasmComponent;
            descriptor_to_json(&descriptor).map_err(|error| error.to_string())
        }

        fn contributions() -> Result<String, String> {
            let contributions = SyncPlugin
                .contributions()
                .map_err(|error| error.to_string())?;
            contributions_to_json(&contributions).map_err(|error| error.to_string())
        }

        fn on_load() -> Result<(), String> {
            Ok(())
        }

        fn on_enable() -> Result<(), String> {
            Ok(())
        }

        fn on_disable() -> Result<(), String> {
            Ok(())
        }

        fn on_unload() -> Result<(), String> {
            Ok(())
        }
    }

    export!(SyncWasm);
}
