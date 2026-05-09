//! Sandboxing policy types for script and plugin execution.
//!
//! This crate intentionally stays small: it only defines serializable policy
//! objects that can be shared by engines, hosts, and higher-level runtimes.

pub mod sandbox;
