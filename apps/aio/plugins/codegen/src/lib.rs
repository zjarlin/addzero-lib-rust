#![forbid(unsafe_code)]

//! 在当前客户机生成 Rust enum 和 struct 源文件的 AIO 插件。

pub mod contract;
pub mod generator;
pub mod plugin;
pub mod routes;
pub mod ui;

pub use plugin::CodegenPlugin;

rudi::enable! {}
