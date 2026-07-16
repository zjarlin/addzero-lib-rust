#![forbid(unsafe_code)]

//! Dioxus UI adapter for AZ AIO.
//!
//! Components are compiled from selected upstream `rust-ui/dioxus-ui` source
//! files pinned under `vendor/rust-ui-dioxus-ui`; keep that checkout unmodified
//! and update it as a submodule when upstream needs to move.

pub mod ui;
