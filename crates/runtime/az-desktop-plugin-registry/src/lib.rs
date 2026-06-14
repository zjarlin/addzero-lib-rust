#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

automod::dir!(pub "src");

// Re-exports required by macros that expand to $crate::Xxx
pub use inventory;

#[doc(hidden)]
pub use az_derive_aliases as __az_desktop_plugin_registry_derive_aliases;

pub use crate::api::{DesktopPluginRegistration, default_desktop_plugin_constructor, load_plugins};
