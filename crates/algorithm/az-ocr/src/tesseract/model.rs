//! Tesseract OCR public models.

use std::collections::HashMap;
use std::path::PathBuf;

/// Tesseract OCR runtime options.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct TesseractOcrOptions {
    /// Tesseract language code, for example `eng` or `chi_sim`.
    pub lang: String,
    /// Tesseract config variables passed as `-c key=value`.
    pub config_variables: HashMap<String, String>,
    /// Input DPI passed through `--dpi`.
    pub dpi: Option<i32>,
    /// Page segmentation mode passed through `--psm`.
    pub psm: Option<i32>,
    /// OCR engine mode passed through `--oem`.
    pub oem: Option<i32>,
}

/// One word-level Tesseract TSV output row.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TesseractOcrWord {
    /// Page number.
    pub page_num: i32,
    /// Block number.
    pub block_num: i32,
    /// Paragraph number.
    pub paragraph_num: i32,
    /// Line number.
    pub line_num: i32,
    /// Word number.
    pub word_num: i32,
    /// Bounding box left coordinate.
    pub left: i32,
    /// Bounding box top coordinate.
    pub top: i32,
    /// Bounding box width.
    pub width: i32,
    /// Bounding box height.
    pub height: i32,
    /// Tesseract confidence score.
    pub confidence: f32,
    /// Recognized word text.
    pub text: String,
}

/// Tesseract OCR run result.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TesseractOcrRun {
    /// Input image path.
    pub input_path: PathBuf,
    /// Plain text output.
    pub recognized_text: String,
    /// Raw TSV output from Tesseract.
    pub raw_tsv: String,
    /// Word-level parsed TSV rows.
    pub words: Vec<TesseractOcrWord>,
}
