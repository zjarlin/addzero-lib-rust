#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

automod::dir!("src");

pub use document::LineCrdtDocument;
pub use error::{LineCrdtError, LineCrdtResult};
pub use wire::{
    LineCrdtImportReport, LineCrdtPendingRange, LineCrdtSnapshot, LineCrdtUpdate, LineCrdtVersion,
};
