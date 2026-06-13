//! High-level Database API and REPL interface.
//!
//! This module provides a clean, user-facing API for the database
//! and an interactive command-line interface.

automod::dir!("src/db");

pub use api::{Database, DatabaseConfig};
pub use connection::{Connection, ConnectionPool};
pub use repl::{Repl, ReplConfig};
