use std::process::Command;
use std::sync::OnceLock;

use anyhow::{Context, Result};

pub fn assert_ffmpeg_exists() -> Result<()> {
    let status = Command::new("ffmpeg")
        .arg("-version")
        .status()
        .context("launch ffmpeg -version")?;

    if !status.success() {
        anyhow::bail!("ffmpeg not available in PATH");
    }

    Ok(())
}

pub fn run_ffmpeg(args: &[String]) -> Result<()> {
    let status = Command::new("ffmpeg")
        .arg("-y")
        .args(args)
        .status()
        .with_context(|| format!("run ffmpeg with args: {args:?}"))?;

    if !status.success() {
        anyhow::bail!("ffmpeg failed with status: {status}");
    }
    Ok(())
}

pub fn filter_exists(name: &str) -> bool {
    static FILTERS: OnceLock<Option<String>> = OnceLock::new();

    let filters = FILTERS.get_or_init(|| {
        let output = Command::new("ffmpeg")
            .arg("-hide_banner")
            .arg("-filters")
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        Some(String::from_utf8_lossy(&output.stdout).into_owned())
    });

    filters
        .as_deref()
        .is_some_and(|filters| filter_list_contains(filters, name))
}

fn filter_list_contains(filters: &str, name: &str) -> bool {
    filters.lines().any(|line| {
        let mut parts = line.split_whitespace();
        let _flags = parts.next();
        matches!(parts.next(), Some(filter_name) if filter_name == name)
    })
}

#[cfg(test)]
mod tests {
    use super::filter_list_contains;

    #[test]
    fn finds_filter_names_without_matching_descriptions() {
        let filters = "\
 TS allpass           A->A       Apply a two-pole all-pass filter.\n\
 .. drawtext          V->V       Draw text on top of video frames.\n";

        assert!(filter_list_contains(filters, "drawtext"));
        assert!(!filter_list_contains(filters, "pass"));
    }
}
