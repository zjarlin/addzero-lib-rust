#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

automod::dir!(pub "src");

pub use plugin::LinuxPlugin;

rudi::enable! {}
