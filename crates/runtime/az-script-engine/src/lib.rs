//! Common contracts for embeddable script engines.
//!
//! This crate focuses on one thing: defining the request/response types and
//! traits that concrete engines such as Rhai can implement.

pub mod script;

pub use script::*;
