#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

mod capability;
mod connection;
mod error;
mod sql;
mod value;

pub use connection::GitDb;
