#![cfg_attr(not(target_arch = "wasm32"), forbid(unsafe_code))]

automod::dir!(pub "src");

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
pub use model::{LowcodeApp, LowcodePage};
#[cfg(not(target_arch = "wasm32"))]
pub use store::LowcodeStore;
