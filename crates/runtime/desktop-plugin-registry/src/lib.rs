#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

automod::dir!(pub "src");

// Re-exports required by macros that expand to $crate::Xxx
pub use inventory;

pub use crate::api::{DesktopPluginRegistration, default_desktop_plugin_constructor, load_plugins};
