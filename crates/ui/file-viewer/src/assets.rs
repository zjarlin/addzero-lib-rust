use dioxus::prelude::{Asset, AssetOptions, asset};

pub(crate) const VIEWER_STYLE: Asset = asset!("/assets/file-viewer.css", AssetOptions::css());
pub(crate) const VIEWER_ENGINE: Asset = asset!("/assets/file-viewer-engine.js", AssetOptions::js());
pub(crate) const MARKED_SCRIPT: Asset = asset!("/assets/vendor/marked.umd.js");
pub(crate) const DOMPURIFY_SCRIPT: Asset = asset!("/assets/vendor/purify.min.js");
pub(crate) const PDF_MODULE: Asset = asset!("/assets/vendor/pdf.min.mjs");
pub(crate) const PDF_WORKER: Asset = asset!("/assets/vendor/pdf.worker.min.mjs");
pub(crate) const JSZIP_SCRIPT: Asset = asset!("/assets/vendor/jszip.min.js");
pub(crate) const DOCX_SCRIPT: Asset = asset!("/assets/vendor/docx-preview.min.js");
