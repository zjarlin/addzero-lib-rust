//! GitDB - A Git-backed Document Database
//!
//! This crate provides a SQL database that uses Git as its storage backend.
//! Every mutation is a commit, every table is a directory, and your entire
//! database history is preserved in `.git/`.
//!
//! # Example
//!
//! ```no_run
//! use gitdb::db::Database;
//!
//! let mut db = Database::open("./my_database").unwrap();
//! db.execute("CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT)").unwrap();
//! db.execute("INSERT INTO users (id, name) VALUES ('1', 'Alice')").unwrap();
//! ```

#![allow(dead_code)] // Many methods are for public API extensibility

pub mod blob_store;
pub mod catalog;
pub mod db;
pub mod executor;
pub mod planner;
pub mod sql;
pub mod storage;
pub mod transaction;
