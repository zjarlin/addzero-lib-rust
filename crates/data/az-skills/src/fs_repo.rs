use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use az_derive_aliases::{apply, deserialize_debug};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::types::{Skill, SkillSource, SkillUpsert};

/// 用于保证关键词渲染幂等的标记。重写 `SKILL.md` 描述时，渲染出的关键词句会被
/// 这两个 HTML 注释包住，后续读取即可精确剥离托管片段，再提取用户手写文本。
const KEYWORDS_START: &str = "<!-- keywords:start -->";
const KEYWORDS_END: &str = "<!-- keywords:end -->";

#[apply(deserialize_debug)]
struct Frontmatter {
    name: Option<String>,
    description: Option<String>,
}

/// 面向 `~/.agents/skills/<name>/SKILL.md` 的文件系统仓库。
pub struct FsRepo {
    root: PathBuf,
}

impl FsRepo {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        Self { root: root.into() }
    }

    /// 默认位置：`$ADDZERO_SKILLS_FS_ROOT`，未设置时使用 `~/.agents/skills`。
    pub fn default_root() -> Result<Self> {
        if let Ok(raw) = std::env::var("ADDZERO_SKILLS_FS_ROOT") {
            return Ok(Self::new(PathBuf::from(raw)));
        }
        let home = std::env::var("HOME").context("HOME env var is not set")?;
        Ok(Self::new(
            PathBuf::from(home).join(".agents").join("skills"),
        ))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// 列出根目录下的所有技能。没有可解析 `SKILL.md` 的目录会被跳过，避免单个坏文件
    /// 让整个 admin 技能面板不可用。
    pub async fn list(&self) -> Result<Vec<Skill>> {
        let mut out = Vec::new();
        if !tokio::fs::try_exists(&self.root).await.unwrap_or(false) {
            return Ok(out);
        }
        let mut dir = tokio::fs::read_dir(&self.root)
            .await
            .with_context(|| format!("read_dir {}", self.root.display()))?;
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            let meta = match tokio::fs::metadata(&path).await {
                Ok(m) => m,
                Err(_) => continue,
            };
            if !meta.is_dir() {
                continue;
            }
            let skill_md = path.join("SKILL.md");
            if !tokio::fs::try_exists(&skill_md).await.unwrap_or(false) {
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
        if !tokio::fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(None);
        }
        Ok(Some(self.read_skill(&path).await?))
    }

    pub async fn delete(&self, name: &str) -> Result<()> {
        let dir = self.root.join(name);
        if tokio::fs::try_exists(&dir).await.unwrap_or(false) {
            tokio::fs::remove_dir_all(&dir)
                .await
                .with_context(|| format!("remove {}", dir.display()))?;
        }
        Ok(())
    }

    /// 原子写入技能：先写 `SKILL.md.tmp`，再重命名覆盖 `SKILL.md`。
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

/// 将 markdown 文件拆成 `(frontmatter_yaml, body)`；没有 frontmatter 时前者为空。
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

/// 从描述中提取 `keywords` 列表。优先读取托管的
/// `<!-- keywords:start --> ... <!-- keywords:end -->` 片段；若不存在，则回退解析
/// 开头的“当用户提到 X、Y、Z 时”句式。返回的描述会剥离托管片段，保证往返写入幂等。
fn extract_keywords_from_description(description: &str) -> (Vec<String>, String) {
    if let (Some(start_idx), Some(end_idx)) = (
        description.find(KEYWORDS_START),
        description.find(KEYWORDS_END),
    ) && start_idx < end_idx
    {
        let inner_start = start_idx + KEYWORDS_START.len();
        let inner = &description[inner_start..end_idx];
        let keywords = parse_keyword_phrase(inner);
        let mut clean = String::with_capacity(description.len());
        clean.push_str(&description[..start_idx]);
        clean.push_str(&description[end_idx + KEYWORDS_END.len()..]);
        return (keywords, clean.trim().to_string());
    }

    if let Some(raw_list) = fallback_keyword_phrase(description) {
        let keywords = split_keywords(raw_list);
        return (keywords, description.trim().to_string());
    }

    (Vec::new(), description.trim().to_string())
}

fn fallback_keyword_phrase(description: &str) -> Option<&str> {
    let after_prefix = description.trim_start().strip_prefix("当用户提到")?.trim_start();
    let (raw_list, _) = after_prefix.split_once('时')?;
    Some(raw_list.trim())
}

fn parse_keyword_phrase(inner: &str) -> Vec<String> {
    let trimmed = inner.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    // 剥离开头的“当用户提到”和结尾的“时...”，只保留关键词列表。
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

/// 将 `SkillUpsert` 渲染回完整的 `SKILL.md` 字符串。
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

/// 渲染描述，并把关键词句放进托管标记之间。没有关键词时会完全移除标记。
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
    ) && start_idx < end_idx
    {
        let mut clean = String::with_capacity(description.len());
        clean.push_str(&description[..start_idx]);
        clean.push_str(&description[end_idx + KEYWORDS_END.len()..]);
        return clean;
    }
    description.to_string()
}

/// 转义 YAML 标量。这里始终加引号，避免冒号、前导标点和 Unicode 内容误触解析规则。
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
