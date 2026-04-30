#[cfg(target_arch = "wasm32")]
use getrandom as _;

pub mod admin;
pub mod app;
pub mod dotfiles_catalog;
pub mod knowledge_catalog;
pub mod package_catalog;
pub mod scenes;
pub mod services;
pub mod state;

#[cfg(not(target_arch = "wasm32"))]
pub mod server;
