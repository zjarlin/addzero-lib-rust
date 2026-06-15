#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

automod::dir!(pub "src");

pub use plugin::DriveCenterPlugin;

rudi::enable! {}
