#![cfg(target_arch = "wasm32")]

use az_aio_plugin_api::api::{AzAioPlugin, PluginKind, contributions_to_json, descriptor_to_json};

use crate::SyncPlugin;

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
