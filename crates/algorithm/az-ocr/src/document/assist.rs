//! Document parsing execution helpers.

use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, bail};

use crate::document::model::{
    DocumentParsingOptions, DocumentParsingOutputFiles, DocumentParsingRun,
};

/// Run PP-StructureV3 document parsing and load normalized Markdown/JSON outputs.
///
/// # Errors
/// Returns an error when the input cannot be resolved, the bridge process fails, or expected
/// normalized output files are missing.
pub fn run_document_parsing_to_markdown_from_path(
    input_path: impl AsRef<Path>,
    options: &DocumentParsingOptions,
) -> anyhow::Result<DocumentParsingRun> {
    let input_path = std::fs::canonicalize(input_path.as_ref()).with_context(|| {
        format!(
            "failed to resolve document parsing input `{}`",
            input_path.as_ref().display()
        )
    })?;
    recreate_dir(&options.output_dir)?;

    let bridge_script = options.resolved_bridge_script();
    if !bridge_script.is_file() {
        bail!(
            "PP-StructureV3 bridge script `{}` is missing",
            bridge_script.display()
        );
    }

    let output = build_bridge_command(&input_path, options, &bridge_script)
        .output()
        .with_context(|| {
            format!(
                "failed to start PP-StructureV3 bridge `{}` with `{}`",
                bridge_script.display(),
                options.python_program.display()
            )
        })?;
    if !output.status.success() {
        bail!(
            "PP-StructureV3 bridge failed with status {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let files = DocumentParsingOutputFiles {
        markdown: options.output_dir.join("document.md"),
        structured_json: options.output_dir.join("structured.json"),
        manifest_json: options.output_dir.join("manifest.json"),
    };
    assert_existing_file(&files.markdown)?;
    assert_existing_file(&files.structured_json)?;
    assert_existing_file(&files.manifest_json)?;
    let markdown = fs::read_to_string(&files.markdown)
        .with_context(|| format!("failed to read markdown `{}`", files.markdown.display()))?;
    let structured_json = fs::read_to_string(&files.structured_json).with_context(|| {
        format!(
            "failed to read structured JSON `{}`",
            files.structured_json.display()
        )
    })?;
    let structured_json = serde_json::from_str(&structured_json).with_context(|| {
        format!(
            "failed to parse structured JSON `{}`",
            files.structured_json.display()
        )
    })?;

    Ok(DocumentParsingRun {
        input_path,
        output_dir: options.output_dir.clone(),
        files,
        markdown,
        structured_json,
    })
}

fn build_bridge_command(
    input_path: &Path,
    options: &DocumentParsingOptions,
    bridge_script: &Path,
) -> Command {
    let mut command = Command::new(&options.python_program);
    command
        .arg(bridge_script)
        .arg("--input")
        .arg(input_path)
        .arg("--output-dir")
        .arg(&options.output_dir)
        .arg("--use-doc-orientation-classify")
        .arg(bool_arg(options.use_doc_orientation_classify))
        .arg("--use-doc-unwarping")
        .arg(bool_arg(options.use_doc_unwarping))
        .arg("--use-textline-orientation")
        .arg(bool_arg(options.use_textline_orientation))
        .arg("--use-table-recognition")
        .arg(bool_arg(options.use_table_recognition))
        .arg("--use-formula-recognition")
        .arg(bool_arg(options.use_formula_recognition))
        .arg("--use-chart-recognition")
        .arg(bool_arg(options.use_chart_recognition))
        .arg("--use-region-detection")
        .arg(bool_arg(options.use_region_detection))
        .args(&options.extra_args);
    command
}

fn bool_arg(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn recreate_dir(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)
            .with_context(|| format!("failed to remove document parser output dir `{}`", path.display()))?;
    }
    fs::create_dir_all(path)
        .with_context(|| format!("failed to create document parser output dir `{}`", path.display()))?;
    Ok(())
}

fn assert_existing_file(path: &Path) -> anyhow::Result<()> {
    if path.is_file() {
        Ok(())
    } else {
        bail!("document parser output file `{}` is missing", path.display())
    }
}
