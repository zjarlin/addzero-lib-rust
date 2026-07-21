use std::path::{Path, PathBuf};

use az_ocr::document::assist::run_document_parsing_to_markdown_from_path;
use az_ocr::document::model::DocumentParsingOptions;
use tempfile::TempDir;

fn fixture_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ocr_text_recognition/input")
        .join(file_name)
}

fn write_fake_pp_structure_runner(path: &Path) -> anyhow::Result<()> {
    std::fs::write(
        path,
        r##"#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("--input", required=True)
parser.add_argument("--output-dir", required=True)
parser.add_argument("--use-doc-orientation-classify")
parser.add_argument("--use-doc-unwarping")
parser.add_argument("--use-textline-orientation")
parser.add_argument("--use-table-recognition")
parser.add_argument("--use-formula-recognition")
parser.add_argument("--use-chart-recognition")
parser.add_argument("--use-region-detection")
args = parser.parse_args()

out = Path(args.output_dir)
out.mkdir(parents=True, exist_ok=True)
markdown = """# 复杂版面解析

正文段落按阅读顺序输出。

| 商品 | 数量 | 金额 |
| --- | ---: | ---: |
| 复杂表格 | 2 | 88 |
"""
(out / "document.md").write_text(markdown, encoding="utf-8")
structured = {
    "engine": "fake-pp-structure-v3",
    "pages": [{
        "parsing_res_list": [
            {"block_label": "text", "block_content": "正文段落按阅读顺序输出。"},
            {"block_label": "table", "block_content": markdown},
        ],
        "table_res_list": [{
            "pred_html": "<table><tr><td>复杂表格</td><td>2</td><td>88</td></tr></table>",
            "cell_box_list": [[[0, 0], [10, 0], [10, 10], [0, 10]]],
        }],
    }],
}
(out / "structured.json").write_text(json.dumps(structured, ensure_ascii=False), encoding="utf-8")
(out / "manifest.json").write_text(json.dumps({"artifact_files": ["document.md", "structured.json"]}), encoding="utf-8")
"##,
    )?;
    Ok(())
}

#[test]
fn document_parser_should_return_markdown_and_structured_table_output() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let runner = temp.path().join("fake_pp_structure.py");
    write_fake_pp_structure_runner(&runner)?;
    let mut options = DocumentParsingOptions::new(temp.path().join("out"));
    options.bridge_script = Some(runner);

    let run = run_document_parsing_to_markdown_from_path(fixture_path("ocr_text.jpg"), &options)?;

    // 关键断言：文档解析链路必须保留表格 Markdown 和结构化表格 JSON。
    assert!(run.markdown.contains("| 商品 | 数量 | 金额 |"));
    assert!(run.markdown.contains("正文段落按阅读顺序输出"));
    assert_eq!(run.structured_json["engine"], "fake-pp-structure-v3");
    assert!(
        run.structured_json["pages"][0]["table_res_list"][0]["pred_html"]
            .as_str()
            .is_some_and(|html| html.contains("<table>"))
    );
    assert!(run.files.markdown.is_file());
    assert!(run.files.structured_json.is_file());
    assert!(run.files.manifest_json.is_file());
    Ok(())
}
