use std::path::PathBuf;

use anyhow::Result;
use tempfile::TempDir;
use tool_project_diagnostics::api::{
    ProjectScanOptions, ProjectSourceFile, SkipReason, SourceLanguage, SyntaxDiagnosticKind,
    scan_project, scan_project_files, scan_project_with_options,
};

#[tokio::test]
async fn scan_project_reports_tree_sitter_syntax_errors() -> Result<()> {
    let project = TempDir::new()?;
    std::fs::write(project.path().join("good.rs"), "fn good() {}\n")?;
    std::fs::write(project.path().join("bad.rs"), "fn bad( {\n")?;
    std::fs::write(project.path().join("bad.ts"), "const answer: = 42\n")?;

    let report = scan_project(project.path()).await?;

    assert_eq!(report.parsed_files, 3);
    assert!(report.diagnostic_count >= 2);

    let rust_report = report
        .files
        .iter()
        .find(|file| file.path == PathBuf::from("bad.rs"))
        .expect("bad.rs should be parsed");
    assert_eq!(rust_report.language, SourceLanguage::Rust);
    assert!(
        rust_report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.kind == SyntaxDiagnosticKind::SyntaxError)
    );

    let serialized = serde_json::to_string(&report)?;
    assert!(serialized.contains("bad.rs"));
    Ok(())
}

#[tokio::test]
async fn scan_uploaded_project_files_without_local_environment() -> Result<()> {
    let report = scan_project_files(
        "upload://project.zip",
        vec![
            ProjectSourceFile {
                path: PathBuf::from("src/main.rs"),
                bytes: b"fn main( {\n".to_vec(),
            },
            ProjectSourceFile {
                path: PathBuf::from("notes.txt"),
                bytes: b"ignored".to_vec(),
            },
        ],
    )
    .await?;

    assert_eq!(report.parsed_files, 1);
    assert_eq!(report.skipped_files, 1);
    assert!(report.diagnostic_count > 0);
    assert_eq!(report.files[0].path, PathBuf::from("src/main.rs"));
    assert!(matches!(
        report.skipped[0].reason,
        SkipReason::UnsupportedLanguage
    ));
    Ok(())
}

#[tokio::test]
async fn scan_project_skips_unsupported_and_large_files() -> Result<()> {
    let project = TempDir::new()?;
    std::fs::write(project.path().join("main.rs"), "fn main() {}\n")?;
    std::fs::write(project.path().join("README.md"), "# unsupported\n")?;
    std::fs::write(project.path().join("big.py"), "print('too large')\n")?;

    let report = scan_project_with_options(
        project.path(),
        ProjectScanOptions {
            max_file_bytes: 8,
            ..ProjectScanOptions::default()
        },
    )
    .await?;

    assert_eq!(report.parsed_files, 0);
    assert_eq!(report.skipped_files, 3);
    assert!(report.skipped.iter().any(|file| {
        file.path == PathBuf::from("README.md") && file.reason == SkipReason::UnsupportedLanguage
    }));
    assert!(report.skipped.iter().any(|file| matches!(
        file.reason,
        SkipReason::TooLarge {
            bytes: _,
            max_bytes: 8
        }
    )));
    Ok(())
}

#[tokio::test]
async fn scan_project_respects_max_files() -> Result<()> {
    let project = TempDir::new()?;
    std::fs::write(project.path().join("a.rs"), "fn a() {}\n")?;
    std::fs::write(project.path().join("b.rs"), "fn b() {}\n")?;

    let report = scan_project_with_options(
        project.path(),
        ProjectScanOptions {
            max_files: Some(1),
            ..ProjectScanOptions::default()
        },
    )
    .await?;

    assert_eq!(report.parsed_files, 1);
    assert!(
        report
            .skipped
            .iter()
            .any(|file| { matches!(file.reason, SkipReason::MaxFilesReached) })
    );
    Ok(())
}

#[tokio::test]
#[ignore = "absolute-path smoke test for local project scans"]
async fn scan_absolute_project_path() -> Result<()> {
    let report = scan_project_with_options(
        "/Users/zjarlin/aio/workspace/zjarlin/addzero-lib-rust",
        ProjectScanOptions {
            max_files: Some(128),
            max_diagnostics_per_file: Some(20),
            ..ProjectScanOptions::default()
        },
    )
    .await?;

    assert!(report.parsed_files > 0);
    Ok(())
}
