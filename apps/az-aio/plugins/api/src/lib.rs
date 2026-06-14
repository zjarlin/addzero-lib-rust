#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

automod::dir!(pub "src");

pub use api::*;

#[cfg(not(target_arch = "wasm32"))]
pub use api::inventory;
