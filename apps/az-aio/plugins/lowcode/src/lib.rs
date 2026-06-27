#![forbid(unsafe_code)]

pub mod plugin;
pub mod routes;
pub mod state;
pub mod ui;

pub use plugin::LowcodePlugin;

rudi::enable! {}
