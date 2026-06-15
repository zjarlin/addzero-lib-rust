#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

automod::dir!(pub "src");

pub use backend::catalog_match::installer_matches_catalog;
pub use plugin::SoftwareCenterPlugin;

rudi::enable! {}
