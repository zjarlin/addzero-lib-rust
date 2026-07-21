//! Tesseract OCR execution helpers.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Context;

use crate::tesseract::model::{TesseractOcrOptions, TesseractOcrRun, TesseractOcrWord};

/// Return languages available to the installed Tesseract binary.
///
/// # Errors
/// Returns an error when `tesseract` is not installed or cannot list languages.
pub fn available_tesseract_languages() -> anyhow::Result<Vec<String>> {
    rusty_tesseract::get_tesseract_langs().context("failed to list Tesseract languages")
}

/// Run Tesseract OCR on an image path.
///
/// # Errors
/// Returns an error when the image cannot be loaded or the system `tesseract` command fails.
pub fn run_tesseract_ocr_from_path(
    image_path: impl AsRef<Path>,
    options: &TesseractOcrOptions,
) -> anyhow::Result<TesseractOcrRun> {
    let image_path = std::fs::canonicalize(image_path.as_ref()).with_context(|| {
        format!(
            "failed to resolve Tesseract OCR image `{}`",
            image_path.as_ref().display()
        )
    })?;
    let image = rusty_tesseract::Image::from_path(image_path.clone()).map_err(|error| {
        anyhow::anyhow!(
            "failed to prepare Tesseract OCR image `{}`: {error}",
            image_path.display()
        )
    })?;
    let args = options.to_rusty_args();
    let recognized_text = rusty_tesseract::image_to_string(&image, &args)
        .context("failed to run Tesseract OCR text output")?;
    let data = rusty_tesseract::image_to_data(&image, &args)
        .context("failed to run Tesseract OCR TSV output")?;
    let words = data
        .data
        .into_iter()
        .filter(|item| item.level == 5 && !item.text.trim().is_empty())
        .map(TesseractOcrWord::from)
        .collect();

    Ok(TesseractOcrRun {
        input_path: image_path,
        recognized_text,
        raw_tsv: data.output,
        words,
    })
}

impl TesseractOcrOptions {
    pub(crate) fn to_rusty_args(&self) -> rusty_tesseract::Args {
        rusty_tesseract::Args {
            lang: self.lang.clone(),
            config_variables: self.config_variables.clone(),
            dpi: self.dpi,
            psm: self.psm,
            oem: self.oem,
        }
    }
}

impl Default for TesseractOcrOptions {
    fn default() -> Self {
        Self {
            lang: "eng".to_owned(),
            config_variables: HashMap::new(),
            dpi: Some(150),
            psm: Some(6),
            oem: Some(3),
        }
    }
}

impl From<rusty_tesseract::Data> for TesseractOcrWord {
    fn from(value: rusty_tesseract::Data) -> Self {
        Self {
            page_num: value.page_num,
            block_num: value.block_num,
            paragraph_num: value.par_num,
            line_num: value.line_num,
            word_num: value.word_num,
            left: value.left,
            top: value.top,
            width: value.width,
            height: value.height,
            confidence: value.conf,
            text: value.text,
        }
    }
}
