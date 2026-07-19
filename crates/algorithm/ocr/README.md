# az-ocr

OCR backends for the addzero ecosystem.

The crate currently exposes two backend families:

- `document`: PP-StructureV3 document parsing bridge for complex layout, tables, and Markdown output.
- `paddle`: the existing local PaddleOCR ONNX detection and recognition path.
- `tesseract`: a wrapper around the system `tesseract` command through `rusty-tesseract`.

`rusty-tesseract` does not ship OCR model weights. It delegates to the installed Tesseract binary and whatever language `.traineddata` files are available in the active `tessdata` directory.

The `document` backend delegates complex table, formula, reading-order, layout, and Markdown conversion to PaddleOCR's official PP-StructureV3 Python API. It standardizes the outputs as:

- `document.md`
- `structured.json`
- `manifest.json`

For production Chinese OCR quality, keep PaddleOCR as the main path. Tesseract is available as a secondary backend for environments where its local `traineddata` packages are acceptable. The intended upgrade path is:

- use PP-OCRv6 or newer PaddleOCR models for text detection and recognition,
- use PP-StructureV3 for complex table/layout/formula/reading-order recovery,
- normalize document parsing outputs through the `document` backend instead of trying to infer tables from plain OCR text lines.
