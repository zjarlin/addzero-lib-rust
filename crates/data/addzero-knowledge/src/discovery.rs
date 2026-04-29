use std::{fs, path::Path};

use deunicode::deunicode;
use sha2::{Digest, Sha256};

use crate::types::{KnowledgeDocument, KnowledgeScan, KnowledgeSourceSpec};

const ALLOWED_EXTENSIONS: &[&str] = &["md", "txt", "org", "rst"];
const MAX_TEXT_BYTES: u64 = 1_500_000;
const BLOCKED_SEGMENTS: &[&str] = &[
    ".git",
    ".github",
    "target",
    "node_modules",
    "vendor",
    "dist",
    "build",
    "release",
    "debug",
    "legal",
    "log",
    "logs",
    "cache",
    "data",
    ".cursor",
    ".ccg-switch",
];
const BLOCKED_FILE_PREFIXES: &[&str] = &[
    "license",
    "changelog",
    "contributing",
    "security",
    "authors",
    "ofl",
    "eula",
    "thirdpartynotices",
    "readme_do_not_touch_files",
    "backers",
    "support",
];
const BLOCKED_FILE_SUBSTRINGS: &[&str] = &[
    ".long-type-",
    "cookie",
    "auth_token",
    "api_key",
    "opentoken",
    "115_cookie",
];

pub fn discover_documents(sources: &[KnowledgeSourceSpec]) -> KnowledgeScan {
    let mut aggregated = KnowledgeScan::default();
    for source in sources {
        let scan = discover_source_documents(source);
        aggregated.documents.extend(scan.documents);
        aggregated.skipped_paths.extend(scan.skipped_paths);
    }
    aggregated.documents.sort_by(
        |left, right| match left.source_name.cmp(&right.source_name) {
            std::cmp::Ordering::Equal => left.relative_path.cmp(&right.relative_path),
            ordering => ordering,
        },
    );
    aggregated
}

pub fn discover_source_documents(source: &KnowledgeSourceSpec) -> KnowledgeScan {
    let mut scan = KnowledgeScan::default();
    if !source.root_path.exists() {
        return scan;
    }

    let mut stack = vec![source.root_path.clone()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => {
                scan.skipped_paths.push(dir.display().to_string());
                continue;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if should_skip_dir(&path) {
                    continue;
                }
                stack.push(path);
                continue;
            }

            if !should_include_file(&path) {
                continue;
            }

            match build_document(source, &path) {
                Some(doc) => scan.documents.push(doc),
                None => scan.skipped_paths.push(path.display().to_string()),
            }
        }
    }

    scan.documents
        .sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    scan
}

fn build_document(source: &KnowledgeSourceSpec, path: &Path) -> Option<KnowledgeDocument> {
    let content = fs::read_to_string(path).ok()?;
    let filename = path.file_name()?.to_string_lossy().to_string();
    let relative_path = path
        .strip_prefix(&source.root_path)
        .ok()?
        .display()
        .to_string();
    let title = extract_title(&content, path);
    let headings = extract_headings(&content);
    let cleaned = clean_text(&content);
    let preview = truncate_chars(&cleaned, 110);
    let excerpt = truncate_chars(&cleaned, 900);
    let content_hash = compute_hash(&content);
    let slug = format!(
        "{}-{}-{}",
        source.slug,
        slugify(&relative_path),
        &content_hash[..8]
    );

    Some(KnowledgeDocument {
        source_slug: source.slug.clone(),
        source_name: source.name.clone(),
        source_root: source.root_path.display().to_string(),
        slug,
        title,
        filename,
        source_path: path.display().to_string(),
        relative_path,
        bytes: content.len(),
        section_count: headings.len(),
        preview,
        excerpt,
        headings,
        body: content,
        content_hash,
    })
}

fn should_skip_dir(path: &Path) -> bool {
    path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        BLOCKED_SEGMENTS.contains(&name.as_str())
            || name.starts_with("jdk-")
            || name.starts_with("python-")
    })
}

fn should_include_file(path: &Path) -> bool {
    if should_skip_dir(path) {
        return false;
    }

    let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
        return false;
    };
    if !ALLOWED_EXTENSIONS
        .iter()
        .any(|allowed| extension.eq_ignore_ascii_case(allowed))
    {
        return false;
    }

    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return false,
    };
    if metadata.len() > MAX_TEXT_BYTES {
        return false;
    }

    let filename = path
        .file_name()
        .map(|item| item.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();

    !BLOCKED_FILE_PREFIXES
        .iter()
        .any(|prefix| filename.starts_with(prefix))
        && !BLOCKED_FILE_SUBSTRINGS
            .iter()
            .any(|fragment| filename.contains(fragment))
}

fn extract_title(content: &str, path: &Path) -> String {
    content
        .lines()
        .find_map(|line| line.strip_prefix("# ").map(str::trim))
        .filter(|title| !title.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            content
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty() && !line.starts_with("```"))
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| {
            path.file_stem()
                .map(|stem| cleanup_stem(&stem.to_string_lossy()))
                .unwrap_or_else(|| "untitled".to_string())
        })
}

fn extract_headings(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            line.strip_prefix("## ")
                .or_else(|| line.strip_prefix("### "))
                .map(str::trim)
        })
        .filter(|heading| !heading.is_empty())
        .map(ToOwned::to_owned)
        .take(10)
        .collect()
}

fn clean_text(content: &str) -> String {
    let mut in_code_block = false;
    let mut lines = Vec::new();

    for raw in content.lines() {
        let line = raw.trim();
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block || line.is_empty() || line.starts_with('#') {
            continue;
        }
        lines.push(
            line.strip_prefix("- ")
                .or_else(|| line.strip_prefix("* "))
                .unwrap_or(line),
        );
    }

    lines.join(" ")
}

fn cleanup_stem(stem: &str) -> String {
    stem.replace(['-', '_'], " ")
}

fn truncate_chars(text: &str, limit: usize) -> String {
    let mut result = String::new();
    for (count, ch) in text.chars().enumerate() {
        if count == limit {
            result.push('…');
            break;
        }
        result.push(ch);
    }
    result
}

fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn slugify(value: &str) -> String {
    let normalized = deunicode(value);
    let mut slug = String::new();
    let mut last_dash = false;

    for ch in normalized.chars() {
        let lowered = ch.to_ascii_lowercase();
        if lowered.is_ascii_alphanumeric() {
            slug.push(lowered);
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }

    let trimmed = slug.trim_matches('-');
    if trimmed.is_empty() {
        "doc".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn markdown_cleanup_keeps_plain_text() {
        let content = "# 标题\n\n## 小节\n- 第一行\n* 第二行\n```rust\nfn main() {}\n```";
        assert_eq!(clean_text(content), "第一行 第二行");
        assert_eq!(extract_headings(content), vec!["小节".to_string()]);
    }

    #[test]
    fn file_filters_skip_vendor_patterns() {
        assert!(should_skip_dir(&PathBuf::from(
            "/tmp/project/target/release"
        )));
        assert!(!should_include_file(&PathBuf::from(
            "/tmp/project/LICENSE.md"
        )));
        assert!(!should_include_file(&PathBuf::from(
            "/tmp/project/mytoken.txt"
        )));
    }
}
