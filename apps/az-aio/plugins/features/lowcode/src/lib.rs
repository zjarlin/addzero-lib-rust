#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]

mod descriptor;

#[cfg(not(target_arch = "wasm32"))]
pub mod api;
#[cfg(not(target_arch = "wasm32"))]
pub mod config;
#[cfg(not(target_arch = "wasm32"))]
pub mod error;
#[cfg(not(target_arch = "wasm32"))]
pub mod model;
#[cfg(not(target_arch = "wasm32"))]
pub mod store;

pub use descriptor::LowcodePlugin;

#[cfg(not(target_arch = "wasm32"))]
pub use api::{LowcodeApiState, lowcode_api_router};
#[cfg(not(target_arch = "wasm32"))]
pub use config::{
    CONFIG_CENTER_BASE_URL_ENV, CONFIG_CENTER_PASSWORD_ENV, CONFIG_CENTER_USERNAME_ENV,
    DATABASE_URL_CONFIG_KEY, DATABASE_URL_ENV, LOWCODE_CONFIG_NAMESPACE, LowcodeConfig,
    LowcodeConfigSource, resolve_lowcode_config,
};
#[cfg(not(target_arch = "wasm32"))]
pub use error::{LowcodeError, LowcodeResult};
#[cfg(not(target_arch = "wasm32"))]
pub use model::{LowcodeApp, LowcodePage};
#[cfg(not(target_arch = "wasm32"))]
pub use store::LowcodeStore;

#[cfg(target_arch = "wasm32")]
mod component {
    use az_aio_plugin_api::{AzAioPlugin, contributions_to_json, descriptor_to_json};

    use super::LowcodePlugin;

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
}
