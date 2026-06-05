#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod document;
mod error;
mod line_index;
mod wire;

pub use document::LineCrdtDocument;
pub use error::{LineCrdtError, LineCrdtResult};
pub use wire::{
    LineCrdtImportReport, LineCrdtPendingRange, LineCrdtSnapshot, LineCrdtUpdate, LineCrdtVersion,
};
