#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

automod::dir!(pub "src");

#[doc(hidden)]
pub use rudi;

pub use crate::registration::{desktop_plugin_provider, load_plugins};
