use std::path::PathBuf;

use az_ocr::tesseract::assist::{available_tesseract_languages, run_tesseract_ocr_from_path};
use az_ocr::tesseract::model::TesseractOcrOptions;

fn fixture_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/tesseract")
        .join(file_name)
}

#[test]
fn tesseract_should_recognize_english_words_when_eng_data_is_available() -> anyhow::Result<()> {
    let Ok(languages) = available_tesseract_languages() else {
        eprintln!("skipping Tesseract smoke test: tesseract command is unavailable");
        return Ok(());
    };
    if !languages.iter().any(|language| language == "eng") {
        eprintln!("skipping Tesseract smoke test: eng.traineddata is unavailable");
        return Ok(());
    }

    let result =
        run_tesseract_ocr_from_path(fixture_path("string.png"), &TesseractOcrOptions::default())?;

    // This checks the wrapper reaches the installed Tesseract engine and parses TSV words.
    assert!(result.recognized_text.contains("LOREM"));
    assert!(result.words.iter().any(|word| word.text == "IPSUM"));
    Ok(())
}
