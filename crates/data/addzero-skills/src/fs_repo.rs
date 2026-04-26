use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::Deserialize;
use uuid::Uuid;

use crate::types::{Skill, SkillSource, SkillUpsert};

/// Markers used to make our keyword-rendering idempotent. When we rewrite a
/// `SKILL.md`'s description, we wrap the rendered keyword sentence with these
/// HTML comments so a subsequent read knows exactly which segment to strip
/// before extracting user-authored text.
const KEYWORDS_START: &str = "<!-- keywords:start -->";
const KEYWORDS_END: &str = "<!-- keywords:end -->";

#[derive(Debug, Deserialize)]
struct Frontmatter {
    name: Option<String>,
    description: Option<String>,
}

/// File-system repository against `~/.agents/skills/<name>/SKILL.md`.
pub struct FsRepo {
    root: PathBuf,
}

impl FsRepo {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    /// Default location: `$ADDZERO_SKILLS_FS_ROOT` or `~/.agents/skills`.
    pub fn default_root() -> Result<Self> {
        if let Ok(raw) = std::env::var("ADDZERO_SKILLS_FS_ROOT") {
            return Ok(Self::new(PathBuf::from(raw)));
        }
        let home = std::env::var("HOME").context("HOME env var is not set")?;
        Ok(Self::new(PathBuf::from(home).join(".agents").join("skills")))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// List all skills under the root. Folders without a parsable `SKILL.md`
    /// are silently ignored so we don't fail the whole admin on a single
    /// malformed file.
    pub async fn list(&self) -> Result<Vec<Skill>> {
        let mut out = Vec::new();
        if !self.root.exists() {
            return Ok(out);
        }
        let mut dir = tokio::fs::read_dir(&self.root)
            .await
            .with_context(|| format!("read_dir {}", self.root.display()))?;
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let skill_md = path.join("SKILL.md");
            if !skill_md.exists() {
                continue;
            }
            match self.read_skill(&skill_md).await {
                Ok(skill) => out.push(skill),
                Err(err) => log::warn!("skip skill {}: {err:?}", path.display()),
            }
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }

    pub async fn get(&self, name: &str) -> Result<Option<Skill>> {
        let path = self.root.join(name).join("SKILL.md");
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(self.read_skill(&path).await?))
    }

    pub async fn delete(&self, name: &str) -> Result<()> {
        let dir = self.root.join(name);
        if dir.exists() {
            tokio::fs::remove_dir_all(&dir)
                .await
                .with_context(|| format!("remove {}", dir.display()))?;
        }
        Ok(())
    }

    /// Write a skill atomically: write `SKILL.md.tmp`, rename over `SKILL.md`.
    pub async fn upsert(&self, input: &SkillUpsert) -> Result<Skill> {
        let dir = self.root.join(&input.name);
        tokio::fs::create_dir_all(&dir)
            .await
            .with_context(|| format!("mkdir {}", dir.display()))?;

        let rendered = render_skill_md(input);
        let final_path = dir.join("SKILL.md");
        let tmp_path = dir.join("SKILL.md.tmp");
        tokio::fs::write(&tmp_path, rendered.as_bytes())
            .await
            .with_context(|| format!("write {}", tmp_path.display()))?;
        tokio::fs::rename(&tmp_path, &final_path)
            .await
            .with_context(|| format!("rename to {}", final_path.display()))?;

        let updated_at = file_updated_at(&final_path)
            .await
            .unwrap_or_else(|_| Utc::now());
        Ok(Skill {
            id: Uuid::new_v4(),
            name: input.name.clone(),
            keywords: input.keywords.clone(),
            description: input.description.clone(),
            body: input.body.clone(),
            content_hash: input.compute_hash(),
            updated_at,
            source: SkillSource::FileSystem,
        })
    }

    async fn read_skill(&self, path: &Path) -> Result<Skill> {
        let raw = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("read {}", path.display()))?;

        let (frontmatter, body) = split_frontmatter(&raw);
        let fm: Frontmatter = if frontmatter.trim().is_empty() {
            Frontmatter {
                name: None,
                description: None,
            }
        } else {
            serde_yaml::from_str(frontmatter)
                .with_context(|| format!("parse frontmatter in {}", path.display()))?
        };

        let folder_name = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());
        let name = fm.name.or(folder_name).unwrap_or_else(|| "unknown".into());
        let raw_description = fm.description.unwrap_or_default();
        let (keywords, description) = extract_keywords_from_description(&raw_description);

        let upsert = SkillUpsert {
            name: name.clone(),
            keywords: keywords.clone(),
            description: description.clone(),
            body: body.to_string(),
        };
        let content_hash = upsert.compute_hash();
        let updated_at = file_updated_at(path).await.unwrap_or_else(|_| Utc::now());

        Ok(Skill {
            id: Uuid::new_v4(),
            name,
            keywords,
            description,
            body: body.to_string(),
            content_hash,
            updated_at,
            source: SkillSource::FileSystem,
        })
    }
}

async fn file_updated_at(path: &Path) -> Result<DateTime<Utc>> {
    let meta = tokio::fs::metadata(path).await?;
    let modified = meta.modified()?;
    Ok(modified.into())
}

/// Split a markdown file into `(frontmatter_yaml, body)`. If there's no
/// frontmatter, `frontmatter_yaml` is empty.
fn split_frontmatter(raw: &str) -> (&str, &str) {
    let trimmed = raw.trim_start_matches('\u{feff}');
    let Some(rest) = trimmed.strip_prefix("---") else {
        return ("", raw);
    };
    let rest = rest.trim_start_matches('\n');
    if let Some(end) = rest.find("\n---") {
        let frontmatter = &rest[..end];
        let after = &rest[end + 4..];
        let after = after.trim_start_matches('\n');
        (frontmatter, after)
    } else {
        ("", raw)
    }
}

/// Pull a `keywords` list out of a description that may contain our managed
/// `<!-- keywords:start --> ... <!-- keywords:end -->` segment, falling back
/// to a regex over the leading "当用户提到 X、Y、Z 时" pattern. The returned
/// description has the managed segment (if any) stripped so that round-tripping
/// is idempotent.
fn extract_keywords_from_description(description: &str) -> (Vec<String>, String) {
    if let (Some(start_idx), Some(end_idx)) = (
        description.find(KEYWORDS_START),
        description.find(KEYWORDS_END),
    ) {
        if start_idx < end_idx {
            let inner_start = start_idx + KEYWORDS_START.len();
            let inner = &description[inner_start..end_idx];
            let keywords = parse_keyword_phrase(inner);
            let mut clean = String::with_capacity(description.len());
            clean.push_str(&description[..start_idx]);
            clean.push_str(&description[end_idx + KEYWORDS_END.len()..]);
            return (keywords, clean.trim().to_string());
        }
    }

    // Fallback: regex over a free-form leading sentence such as
    //   "当用户提到 a、b、c 时使用..."
    let re = Regex::new(r"当用户提到\s*([^，。\n]+?)\s*时").expect("static regex");
    if let Some(caps) = re.captures(description) {
        let raw_list = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let keywords = split_keywords(raw_list);
        return (keywords, description.trim().to_string());
    }

    (Vec::new(), description.trim().to_string())
}

fn parse_keyword_phrase(inner: &str) -> Vec<String> {
    let trimmed = inner.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    // Strip a leading "当用户提到 " prefix and a trailing " 时...".
    let after_prefix = trimmed
        .strip_prefix("当用户提到")
        .map(str::trim_start)
        .unwrap_or(trimmed);
    let list_part = after_prefix
        .split_once('时')
        .map(|(left, _)| left)
        .unwrap_or(after_prefix);
    split_keywords(list_part)
}

fn split_keywords(raw: &str) -> Vec<String> {
    raw.split(['、', ',', '，', '/'])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// Render a `SkillUpsert` back to a complete `SKILL.md` string.
fn render_skill_md(input: &SkillUpsert) -> String {
    let description = render_description_with_keywords(&input.description, &input.keywords);
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("name: {}\n", yaml_escape_scalar(&input.name)));
    out.push_str(&format!(
        "description: {}\n",
        yaml_escape_scalar(&description)
    ));
    out.push_str("---\n\n");
    out.push_str(input.body.trim_start_matches('\n'));
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Render description so that the keywords sentence lives between our markers.
/// If there are no keywords, we drop the markers entirely.
pub fn render_description_with_keywords(description: &str, keywords: &[String]) -> String {
    let cleaned = strip_managed_block(description).trim().to_string();
    if keywords.is_empty() {
        return cleaned;
    }
    let phrase = format!("当用户提到 {} 时使用。", keywords.join("、"));
    let block = format!("{KEYWORDS_START}{phrase}{KEYWORDS_END}");
    if cleaned.is_empty() {
        block
    } else {
        format!("{block} {cleaned}")
    }
}

fn strip_managed_block(description: &str) -> String {
    if let (Some(start_idx), Some(end_idx)) = (
        description.find(KEYWORDS_START),
        description.find(KEYWORDS_END),
    ) {
        if start_idx < end_idx {
            let mut clean = String::with_capacity(description.len());
            clean.push_str(&description[..start_idx]);
            clean.push_str(&description[end_idx + KEYWORDS_END.len()..]);
            return clean;
        }
    }
    description.to_string()
}

/// Quote YAML scalars when needed. We always quote so that colons, leading
/// punctuation and unicode never trip the parser.
fn yaml_escape_scalar(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_managed_block() {
        let desc = format!(
            "{KEYWORDS_START}当用户提到 ui、frontend 时使用。{KEYWORDS_END} 后面的真实说明。",
        );
        let (keywords, clean) = extract_keywords_from_description(&desc);
        assert_eq!(keywords, vec!["ui".to_string(), "frontend".to_string()]);
        assert_eq!(clean, "后面的真实说明。");
    }

    #[test]
    fn extract_legacy_phrase() {
        let desc = "当用户提到 ui、前端、frontend 时使用。后面是说明。";
        let (keywords, _) = extract_keywords_from_description(desc);
        assert_eq!(keywords, vec!["ui", "前端", "frontend"]);
    }

    #[test]
    fn render_round_trip() {
        let input = SkillUpsert {
            name: "demo".into(),
            keywords: vec!["a".into(), "b".into()],
            description: "原说明".into(),
            body: "正文\n".into(),
        };
        let rendered = render_skill_md(&input);
        assert!(rendered.contains(KEYWORDS_START));
        assert!(rendered.contains(KEYWORDS_END));
        let (front, body) = split_frontmatter(&rendered);
        assert!(front.contains("name: \"demo\""));
        assert_eq!(body.trim(), "正文");
    }
}
