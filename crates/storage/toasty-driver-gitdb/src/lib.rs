#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Toasty driver for `gitdb`.
//!
//! This adapter lets Toasty issue SQL-oriented operations against the local
//! `gitdb` crate. It focuses on the subset `gitdb` already supports well:
//! table creation, inserts, updates, deletes, selects, transactions, and
//! simple migration replay.

mod capability;
mod connection;
mod error;
mod sql;
mod value;

pub use connection::GitDb;
