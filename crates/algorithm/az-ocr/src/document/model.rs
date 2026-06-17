//! Document parsing public models.

use std::path::PathBuf;

/// PP-StructureV3 document parser execution options.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentParsingOptions {
    /// Directory where normalized parser artifacts are written.
    pub output_dir: PathBuf,
    /// Python executable used to run the bridge script.
    pub python_program: PathBuf,
    /// Bridge script path. Defaults to this crate's bundled PP-StructureV3 bridge.
    pub bridge_script: Option<PathBuf>,
    /// Enable document orientation classification.
    pub use_doc_orientation_classify: bool,
    /// Enable document unwarping.
    pub use_doc_unwarping: bool,
    /// Enable text line orientation classification.
    pub use_textline_orientation: bool,
    /// Enable table recognition.
    pub use_table_recognition: bool,
    /// Enable formula recognition.
    pub use_formula_recognition: bool,
    /// Enable chart recognition.
    pub use_chart_recognition: bool,
    /// Enable region detection.
    pub use_region_detection: bool,
    /// Extra raw arguments passed to the bridge script.
    pub extra_args: Vec<String>,
}

impl DocumentParsingOptions {
    /// Build default PP-StructureV3 options for an output directory.
    #[must_use]
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            output_dir: output_dir.into(),
            python_program: PathBuf::from("python3"),
            bridge_script: None,
            use_doc_orientation_classify: false,
            use_doc_unwarping: false,
            use_textline_orientation: false,
            use_table_recognition: true,
            use_formula_recognition: true,
            use_chart_recognition: false,
            use_region_detection: true,
            extra_args: Vec::new(),
        }
    }

    /// Resolve the bridge script used for execution.
    #[must_use]
    pub fn resolved_bridge_script(&self) -> PathBuf {
        self.bridge_script
            .clone()
            .unwrap_or_else(default_pp_structure_v3_bridge_script)
    }
}

/// Normalized document parser output files.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct DocumentParsingOutputFiles {
    /// Concatenated Markdown output.
    pub markdown: PathBuf,
    /// Structured JSON output.
    pub structured_json: PathBuf,
    /// Artifact manifest JSON output.
    pub manifest_json: PathBuf,
}

/// Document parser run result.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct DocumentParsingRun {
    /// Input document image or PDF path.
    pub input_path: PathBuf,
    /// Output directory.
    pub output_dir: PathBuf,
    /// Normalized output files.
    pub files: DocumentParsingOutputFiles,
    /// Markdown content read from `files.markdown`.
    pub markdown: String,
    /// Structured JSON content read from `files.structured_json`.
    pub structured_json: serde_json::Value,
}

fn default_pp_structure_v3_bridge_script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/pp_structure_v3_bridge.py")
}
