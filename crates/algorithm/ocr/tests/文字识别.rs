use std::path::{Path, PathBuf};

use az_ocr::paddle::assist::run_ocr_text_recognition_from_path_with_output;

fn workspace_root() -> PathBuf {
    std::fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."))
        .expect("workspace 根目录必须存在")
}

fn fixture_path(file_name: &str) -> PathBuf {
    std::fs::canonicalize(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/ocr_text_recognition/input")
            .join(file_name),
    )
    .expect("测试输入图片必须存在")
}

fn output_dir(name: &str) -> PathBuf {
    workspace_root()
        .join("target/az-algorithm-results")
        .join(name)
}

fn assert_existing_file(path: &Path) {
    assert!(path.is_file(), "输出文件必须存在：{}", path.display());
}

#[test]
fn ocr_text_recognition_should_return_text_from_input_image() -> anyhow::Result<()> {
    // 输入图片：crates/algorithm/ocr/tests/fixtures/ocr_text_recognition/input/ocr_text.jpg
    //
    // 输出：
    // target/az-algorithm-results/ocr_text_recognition/recognized_text.txt
    // target/az-algorithm-results/ocr_text_recognition/recognized_text.json
    let result = run_ocr_text_recognition_from_path_with_output(
        fixture_path("ocr_text.jpg"),
        output_dir("ocr_text_detection"),
        output_dir("ocr_text_recognition"),
    )?;
    println!("{}", result.recognized_text);

    // 关键断言：OCR 不能只停留在 raw tensor，必须产出可消费的文本结果。
    assert!(!result.recognized_text.trim().is_empty());
    assert!(!result.tokens.is_empty());
    assert!(
        result.recognized_text.contains("22元"),
        "OCR 文本必须包含测试图中的价格片段，实际为：{}",
        result.recognized_text
    );
    assert!(
        result.recognized_text.contains("1000瓶起订"),
        "OCR 文本必须包含测试图中的起订量片段，实际为：{}",
        result.recognized_text
    );
    assert!(
        result.recognized_text.contains("含量"),
        "OCR 文本必须包含测试图中的字段名片段，实际为：{}",
        result.recognized_text
    );
    assert!(
        result.lines.iter().any(|line| !line.text.trim().is_empty()),
        "OCR 必须至少返回一个非空文本行"
    );
    assert_existing_file(&result.files.recognized_text);
    assert_existing_file(&result.files.recognized_text_json);
    assert_eq!(
        std::fs::read_to_string(&result.files.recognized_text)?,
        result.recognized_text
    );
    Ok(())
}
