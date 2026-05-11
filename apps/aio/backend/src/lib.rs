#[cfg(not(target_arch = "wasm32"))]
pub mod auth_cli;
pub mod cli;
pub mod cli_metadata;
pub mod dotfiles_catalog;
#[cfg(not(target_arch = "wasm32"))]
pub mod external_cli;
pub mod knowledge_catalog;
pub mod package_catalog;
pub mod services;
pub mod system_cli;

#[cfg(not(target_arch = "wasm32"))]
pub mod server;
