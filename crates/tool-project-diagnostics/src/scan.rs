//! Public project diagnostics API.
//!
//! The main entry point is [`scan_project`]. Web handlers can unpack an upload
//! into a temporary directory and call this same function without relying on the
//! user's IDE, language server, compiler, or package manager.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::scanner;

/// Configuration for a tree-sitter project scan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectScanOptions {
    /// Follow symlinks while walking the uploaded or local project tree.
    pub follow_symlinks: bool,
    /// Respect `.gitignore`, `.ignore`, and hidden-file filters while walking.
    pub respect_ignore_files: bool,
    /// Maximum file size to parse in bytes.
    pub max_file_bytes: u64,
    /// Maximum number of files to parse. `None` scans every supported file.
    pub max_files: Option<usize>,
    /// Maximum number of diagnostics to keep per file. `None` keeps all.
    pub max_diagnostics_per_file: Option<usize>,
}

impl Default for ProjectScanOptions {
    fn default() -> Self {
        Self {
            follow_symlinks: false,
            respect_ignore_files: true,
            max_file_bytes: 2 * 1024 * 1024,
            max_files: None,
            max_diagnostics_per_file: Some(200),
        }
    }
}

/// Full diagnostics report for one project scan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectDiagnosticReport {
    /// Absolute or caller-provided project root that was scanned.
    pub root: PathBuf,
    /// UTC-ish system timestamp captured at scan start.
    pub scanned_at: SystemTime,
    /// Number of supported source files parsed.
    pub parsed_files: usize,
    /// Number of files skipped before parsing.
    pub skipped_files: usize,
    /// Total diagnostics emitted across all files.
    pub diagnostic_count: usize,
    /// Per-file syntax diagnostics.
    pub files: Vec<FileDiagnosticReport>,
    /// Files that were skipped and the reason they were skipped.
    pub skipped: Vec<SkippedFile>,
}

impl ProjectDiagnosticReport {
    /// Returns `true` when tree-sitter reported no syntax diagnostics.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.diagnostic_count == 0
    }
}

/// Uploaded or otherwise in-memory project file for syntax diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSourceFile {
    /// Source path relative to the project root or archive root.
    pub path: PathBuf,
    /// Raw file contents.
    pub bytes: Vec<u8>,
}

/// Diagnostics for one parsed source file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileDiagnosticReport {
    /// Source path relative to the scanned project root when possible.
    pub path: PathBuf,
    /// Detected tree-sitter language.
    pub language: SourceLanguage,
    /// File size in bytes.
    pub bytes: u64,
    /// Syntax diagnostics found in this file.
    pub diagnostics: Vec<SyntaxDiagnostic>,
}

/// A file skipped before tree-sitter parsing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkippedFile {
    /// Source path relative to the scanned project root when possible.
    pub path: PathBuf,
    /// Why the file was skipped.
    pub reason: SkipReason,
}

/// Reason a file was not parsed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkipReason {
    /// File extension is not mapped to a bundled tree-sitter language.
    UnsupportedLanguage,
    /// File exceeded [`ProjectScanOptions::max_file_bytes`].
    TooLarge { bytes: u64, max_bytes: u64 },
    /// The scan reached [`ProjectScanOptions::max_files`].
    MaxFilesReached,
    /// The file could not be read.
    ReadFailed { message: String },
    /// Tree-sitter could not parse the file.
    ParseFailed { message: String },
}

/// Source languages currently bundled by this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceLanguage {
    /// Rust source.
    Rust,
    /// JavaScript source, including JSX when the extension is `.jsx`.
    JavaScript,
    /// TypeScript source.
    TypeScript,
    /// TypeScript JSX source.
    Tsx,
    /// Python source.
    Python,
    /// Java source.
    Java,
}

/// A syntax issue reported from the tree-sitter parse tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxDiagnostic {
    /// Stable diagnostic category.
    pub kind: SyntaxDiagnosticKind,
    /// Tree-sitter node kind such as `ERROR` or the expected missing node kind.
    pub node_kind: String,
    /// UTF-8 byte range in the source file.
    pub byte_range: ByteRange,
    /// One-based source range for UI display.
    pub range: SourceRange,
    /// Short source excerpt around the diagnostic.
    pub snippet: String,
    /// Human-readable diagnostic message.
    pub message: String,
}

/// Tree-sitter diagnostic category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyntaxDiagnosticKind {
    /// The parser emitted an explicit syntax error node.
    SyntaxError,
    /// The parser recovered by inserting a missing grammar node.
    MissingNode,
}

/// UTF-8 byte range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ByteRange {
    /// Inclusive start byte.
    pub start: usize,
    /// Exclusive end byte.
    pub end: usize,
}

/// One-based source range for editor or web display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRange {
    /// One-based start line.
    pub start_line: usize,
    /// One-based start column in UTF-8 bytes.
    pub start_column: usize,
    /// One-based end line.
    pub end_line: usize,
    /// One-based end column in UTF-8 bytes.
    pub end_column: usize,
}

/// Asynchronously scan a project directory for tree-sitter syntax errors.
///
/// This API is suitable for web backends: unpack the uploaded project into a
/// temporary directory, then call this function and serialize the returned
/// [`ProjectDiagnosticReport`] as the response body.
pub async fn scan_project(root: impl AsRef<Path>) -> Result<ProjectDiagnosticReport> {
    scan_project_with_options(root, ProjectScanOptions::default()).await
}

/// Asynchronously scan a project directory with explicit options.
pub async fn scan_project_with_options(
    root: impl AsRef<Path>,
    options: ProjectScanOptions,
) -> Result<ProjectDiagnosticReport> {
    scanner::scan_project(root.as_ref().to_path_buf(), options).await
}

/// Asynchronously scan already-uploaded project files.
///
/// This avoids any dependency on the user's local development environment and
/// lets a web backend analyze files after unpacking an upload in memory. The
/// supplied `root_label` is only used as the report root, for example
/// `"upload://project.zip"`.
pub async fn scan_project_files(
    root_label: impl Into<PathBuf>,
    files: Vec<ProjectSourceFile>,
) -> Result<ProjectDiagnosticReport> {
    scan_project_files_with_options(root_label, files, ProjectScanOptions::default()).await
}

/// Asynchronously scan already-uploaded project files with explicit options.
pub async fn scan_project_files_with_options(
    root_label: impl Into<PathBuf>,
    files: Vec<ProjectSourceFile>,
    options: ProjectScanOptions,
) -> Result<ProjectDiagnosticReport> {
    scanner::scan_project_files(root_label.into(), files, options).await
}
