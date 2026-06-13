#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

automod::dir!("src");

pub use connection::GitDb;
