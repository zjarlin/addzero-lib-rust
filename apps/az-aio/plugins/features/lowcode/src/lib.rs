#![forbid(unsafe_code)]

automod::dir!(pub "src");

pub use descriptor::LowcodePlugin;

pub use api::{lowcode_api_router, LowcodeApiState};
pub use config::{resolve_lowcode_config, LowcodeConfig, DATABASE_URL_ENV};
pub use model::{LowcodeApp, LowcodePage};
pub use store::LowcodeStore;
