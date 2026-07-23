#![forbid(unsafe_code)]
#![allow(non_snake_case)]

mod assets;
mod component;
mod source;

pub use component::{FileViewer, FileViewerProps, FileViewerTheme};
pub use source::FileViewerKind;
