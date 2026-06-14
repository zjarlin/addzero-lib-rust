#![forbid(unsafe_code)]

automod::dir!(pub "src");

pub use descriptor::LowcodePlugin;

pub use api::{lowcode_api_router, LowcodeApiState};
pub use config::{
    resolve_lowcode_config, LowcodeConfig, LowcodeConfigSource, CONFIG_CENTER_BASE_URL_ENV,
    CONFIG_CENTER_PASSWORD_ENV, CONFIG_CENTER_USERNAME_ENV, DATABASE_URL_CONFIG_KEY,
    DATABASE_URL_ENV, LOWCODE_CONFIG_NAMESPACE,
};
pub use model::{LowcodeApp, LowcodePage};
pub use store::LowcodeStore;
