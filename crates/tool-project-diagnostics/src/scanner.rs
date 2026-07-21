use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};
use ignore::WalkBuilder;
use tokio::task;
use tree_sitter::{Node, Parser, Point};

use crate::scan::{
    ByteRange, FileDiagnosticReport, ProjectDiagnosticReport, ProjectScanOptions, SkipReason,
    ProjectSourceFile, SkippedFile, SourceRange, SyntaxDiagnostic, SyntaxDiagnosticKind,
};
use crate::scan::SourceLanguage;

pub(crate) async fn scan_project(
    root: PathBuf,
    options: ProjectScanOptions,
) -> Result<ProjectDiagnosticReport> {
    let scanned_at = SystemTime::now();
    let root = root
        .canonicalize()
        .with_context(|| format!("failed to resolve project root {}", root.display()))?;
    let (files, mut skipped) = collect_scan_targets(&root, &options)?;

    let target_count = files.len();
    let mut tasks = Vec::with_capacity(target_count);
    for file in files {
        let options = options.clone();
        let root = root.clone();
        tasks.push(task::spawn_blocking(move || parse_file(&root, file, &options)));
    }

    let mut reports = Vec::with_capacity(target_count);
    for task in tasks {
        match task.await.context("project diagnostic worker failed")? {
            Ok(report) => reports.push(report),
            Err(skipped_file) => skipped.push(skipped_file),
        }
    }

    Ok(build_report(root, scanned_at, reports, skipped))
}

pub(crate) async fn scan_project_files(
    root: PathBuf,
    files: Vec<ProjectSourceFile>,
    options: ProjectScanOptions,
) -> Result<ProjectDiagnosticReport> {
    let scanned_at = SystemTime::now();
    let (files, mut skipped) = collect_uploaded_targets(files, &options);

    let target_count = files.len();
    let mut tasks = Vec::with_capacity(target_count);
    for file in files {
        let options = options.clone();
        tasks.push(task::spawn_blocking(move || parse_source(file, &options)));
    }

    let mut reports = Vec::with_capacity(target_count);
    for task in tasks {
        match task.await.context("project diagnostic worker failed")? {
            Ok(report) => reports.push(report),
            Err(skipped_file) => skipped.push(skipped_file),
        }
    }

    Ok(build_report(root, scanned_at, reports, skipped))
}

fn build_report(
    root: PathBuf,
    scanned_at: SystemTime,
    mut reports: Vec<FileDiagnosticReport>,
    mut skipped: Vec<SkippedFile>,
) -> ProjectDiagnosticReport {
    reports.sort_by(|left, right| left.path.cmp(&right.path));
    skipped.sort_by(|left, right| left.path.cmp(&right.path));

    let parsed_files = reports.len();
    let diagnostic_count = reports
        .iter()
        .map(|report| report.diagnostics.len())
        .sum::<usize>();

    ProjectDiagnosticReport {
        root,
        scanned_at,
        parsed_files,
        skipped_files: skipped.len(),
        diagnostic_count,
        files: reports,
        skipped,
    }
}

fn collect_scan_targets(
    root: &Path,
    options: &ProjectScanOptions,
) -> Result<(Vec<ScanTarget>, Vec<SkippedFile>)> {
    let mut walker = WalkBuilder::new(root);
    walker.follow_links(options.follow_symlinks);

    if !options.respect_ignore_files {
        walker.standard_filters(false);
        walker.hidden(false);
        walker.ignore(false);
        walker.git_ignore(false);
        walker.git_exclude(false);
        walker.git_global(false);
    }

    let mut files = Vec::new();
    let mut skipped = Vec::new();
    let mut reached_max_files = false;

    for entry in walker.build() {
        let entry = entry.with_context(|| format!("failed to walk project root {}", root.display()))?;
        let path = entry.path();
        if !entry
            .file_type()
            .map(|file_type| file_type.is_file())
            .unwrap_or(false)
        {
            continue;
        }

        let relative_path = relative_path(root, path);
        let Some(language) = SourceLanguage::from_path(path) else {
            skipped.push(SkippedFile {
                path: relative_path,
                reason: SkipReason::UnsupportedLanguage,
            });
            continue;
        };

        if reached_max_files {
            skipped.push(SkippedFile {
                path: relative_path,
                reason: SkipReason::MaxFilesReached,
            });
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(error) => {
                skipped.push(SkippedFile {
                    path: relative_path,
                    reason: SkipReason::ReadFailed {
                        message: error.to_string(),
                    },
                });
                continue;
            }
        };
        let bytes = metadata.len();
        if bytes > options.max_file_bytes {
            skipped.push(SkippedFile {
                path: relative_path,
                reason: SkipReason::TooLarge {
                    bytes,
                    max_bytes: options.max_file_bytes,
                },
            });
            continue;
        }

        files.push(ScanTarget {
            absolute_path: path.to_path_buf(),
            relative_path,
            language,
            bytes,
        });

        if options.max_files.is_some_and(|max_files| files.len() >= max_files) {
            reached_max_files = true;
        }
    }

    Ok((files, skipped))
}

fn collect_uploaded_targets(
    files: Vec<ProjectSourceFile>,
    options: &ProjectScanOptions,
) -> (Vec<InMemoryScanTarget>, Vec<SkippedFile>) {
    let mut targets = Vec::new();
    let mut skipped = Vec::new();
    let mut reached_max_files = false;

    for file in files {
        let Some(language) = SourceLanguage::from_path(&file.path) else {
            skipped.push(SkippedFile {
                path: file.path,
                reason: SkipReason::UnsupportedLanguage,
            });
            continue;
        };

        if reached_max_files {
            skipped.push(SkippedFile {
                path: file.path,
                reason: SkipReason::MaxFilesReached,
            });
            continue;
        }

        let bytes = file.bytes.len() as u64;
        if bytes > options.max_file_bytes {
            skipped.push(SkippedFile {
                path: file.path,
                reason: SkipReason::TooLarge {
                    bytes,
                    max_bytes: options.max_file_bytes,
                },
            });
            continue;
        }

        targets.push(InMemoryScanTarget {
            path: file.path,
            source: file.bytes,
            language,
            bytes,
        });

        if options
            .max_files
            .is_some_and(|max_files| targets.len() >= max_files)
        {
            reached_max_files = true;
        }
    }

    (targets, skipped)
}

fn parse_file(
    _root: &Path,
    file: ScanTarget,
    options: &ProjectScanOptions,
) -> std::result::Result<FileDiagnosticReport, SkippedFile> {
    let source = std::fs::read(&file.absolute_path).map_err(|error| SkippedFile {
        path: file.relative_path.clone(),
        reason: SkipReason::ReadFailed {
            message: error.to_string(),
        },
    })?;

    parse_source(
        InMemoryScanTarget {
            path: file.relative_path,
            source,
            language: file.language,
            bytes: file.bytes,
        },
        options,
    )
}

fn parse_source(
    file: InMemoryScanTarget,
    options: &ProjectScanOptions,
) -> std::result::Result<FileDiagnosticReport, SkippedFile> {
    let mut parser = Parser::new();
    parser
        .set_language(&file.language.tree_sitter_language())
        .map_err(|error| SkippedFile {
            path: file.path.clone(),
            reason: SkipReason::ParseFailed {
                message: error.to_string(),
            },
        })?;

    let tree = parser.parse(&file.source, None).ok_or_else(|| SkippedFile {
        path: file.path.clone(),
        reason: SkipReason::ParseFailed {
            message: "tree-sitter returned no parse tree".to_string(),
        },
    })?;

    let mut diagnostics = Vec::new();
    if tree.root_node().has_error() {
        collect_diagnostics(
            tree.root_node(),
            &file.source,
            &mut diagnostics,
            options.max_diagnostics_per_file,
        );
    }

    Ok(FileDiagnosticReport {
        path: file.path,
        language: file.language,
        bytes: file.bytes,
        diagnostics,
    })
}

fn collect_diagnostics(
    node: Node<'_>,
    source: &[u8],
    diagnostics: &mut Vec<SyntaxDiagnostic>,
    max_diagnostics: Option<usize>,
) {
    if max_diagnostics.is_some_and(|max| diagnostics.len() >= max) {
        return;
    }

    if node.is_error() {
        diagnostics.push(build_diagnostic(
            SyntaxDiagnosticKind::SyntaxError,
            node,
            source,
        ));
    } else if node.is_missing() {
        diagnostics.push(build_diagnostic(
            SyntaxDiagnosticKind::MissingNode,
            node,
            source,
        ));
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_diagnostics(child, source, diagnostics, max_diagnostics);
        if max_diagnostics.is_some_and(|max| diagnostics.len() >= max) {
            break;
        }
    }
}

fn build_diagnostic(
    kind: SyntaxDiagnosticKind,
    node: Node<'_>,
    source: &[u8],
) -> SyntaxDiagnostic {
    let start = node.start_position();
    let end = node.end_position();
    let node_kind = node.kind().to_string();
    let range = SourceRange {
        start_line: start.row + 1,
        start_column: start.column + 1,
        end_line: end.row + 1,
        end_column: end.column + 1,
    };

    let message = match kind {
        SyntaxDiagnosticKind::SyntaxError => {
            format!("syntax error near {}", display_position(start))
        }
        SyntaxDiagnosticKind::MissingNode => {
            format!(
                "missing `{}` near {}",
                node_kind.trim_start_matches("MISSING "),
                display_position(start)
            )
        }
    };

    SyntaxDiagnostic {
        kind,
        node_kind,
        byte_range: ByteRange {
            start: node.start_byte(),
            end: node.end_byte(),
        },
        range,
        snippet: snippet_for_node(source, node),
        message,
    }
}

fn snippet_for_node(source: &[u8], node: Node<'_>) -> String {
    if node.start_byte() < node.end_byte() {
        return source
            .get(node.start_byte()..node.end_byte())
            .map(String::from_utf8_lossy)
            .unwrap_or_default()
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .chars()
            .take(160)
            .collect();
    }

    String::from_utf8_lossy(source)
        .lines()
        .nth(node.start_position().row)
        .unwrap_or_default()
        .trim()
        .chars()
        .take(160)
        .collect()
}

fn display_position(point: Point) -> String {
    format!("line {}, column {}", point.row + 1, point.column + 1)
}

fn relative_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

#[derive(Debug, Clone)]
struct ScanTarget {
    absolute_path: PathBuf,
    relative_path: PathBuf,
    language: SourceLanguage,
    bytes: u64,
}

#[derive(Debug, Clone)]
struct InMemoryScanTarget {
    path: PathBuf,
    source: Vec<u8>,
    language: SourceLanguage,
    bytes: u64,
}
