//! High-level Database API and REPL interface.
//!
//! This module provides a clean, user-facing API for the database
//! and an interactive command-line interface.

mod api;
mod connection;
mod repl;

pub use api::{Database, DatabaseConfig, DatabaseError, DatabaseResult};
pub use connection::{Connection, ConnectionPool};
pub use repl::{Repl, ReplConfig};
