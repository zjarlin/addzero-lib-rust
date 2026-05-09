#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]

//! Industrial protocol conversion gateway plugin.
//!
//! The current AIO plugin host loads business plugins from `.azplugin` bundles
//! and calls lifecycle exports from the packaged WebAssembly module. The domain
//! model in this crate keeps the gateway routes explicit while the UI surface is
//! contributed through `plugin.toml`.

pub mod domain;

pub use domain::{ConversionRoute, Endpoint, EndpointRole, GatewayProfile, Protocol};

/// Returns the default protocol conversion profile shipped with the plugin.
pub fn default_gateway_profile() -> GatewayProfile<'static> {
    domain::default_profile()
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn aio_on_load() -> i32 {
    0
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn aio_on_enable() -> i32 {
    0
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn aio_on_disable() -> i32 {
    0
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn aio_on_unload() -> i32 {
    0
}
