pub mod cli;
pub mod dotfiles_catalog;
pub mod knowledge_catalog;
pub mod package_catalog;
pub mod services;

#[cfg(not(target_arch = "wasm32"))]
pub mod server;
