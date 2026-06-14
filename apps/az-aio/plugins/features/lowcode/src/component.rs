#![cfg(target_arch = "wasm32")]

use az_aio_plugin_api::api::{AzAioPlugin, contributions_to_json, descriptor_to_json};

use crate::LowcodePlugin;

wit_bindgen::generate!({
    path: "../../wit",
    world: "az-aio-plugin",
});

struct LowcodeWasm;

impl Guest for LowcodeWasm {
    fn describe() -> Result<String, String> {
        descriptor_to_json(&LowcodePlugin.descriptor()).map_err(|error| error.to_string())
    }

    fn contributions() -> Result<String, String> {
        let contributions = LowcodePlugin
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

export!(LowcodeWasm);
