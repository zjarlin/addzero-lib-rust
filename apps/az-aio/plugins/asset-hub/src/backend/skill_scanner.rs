use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};
use az_str::sanitize::to_slug;

const BUILT_IN_TAGS: &[&str] = &[
    "gradle",
    "rust",
    "compose_multiplatform",
    "koin",
    "maven",
    "ksp",
    "docker",
    "ktor",
    "ui",
    "api",
];

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedSkillAsset {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub asset_type: String,
    pub source: String,
    pub origin: String,
    pub tags: Vec<String>,
    pub content: String,
    pub status: String,
    pub md5: Option<String>,
    pub systems: Vec<String>,
}

pub fn scan_skill_assets() -> anyhow::Result<Vec<ScannedSkillAsset>> {
    scan_skill_assets_at(&skill_root()?)
}

pub(crate) fn scan_skill_assets_at(root: &Path) -> anyhow::Result<Vec<ScannedSkillAsset>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut assets = fs::read_dir(root)
        .with_context(|| format!("read skill dir: {}", root.display()))?
        .filter_map(Result::ok)
        .filter_map(|entry| scan_skill_dir(&entry.path()).transpose())
        .collect::<anyhow::Result<Vec<_>>>()?;
    assets.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(assets)
}

fn scan_skill_dir(path: &Path) -> anyhow::Result<Option<ScannedSkillAsset>> {
    let skill_path = path.join("SKILL.md");
    if !skill_path.is_file() {
        return Ok(None);
    }

    let content = fs::read_to_string(&skill_path)
        .with_context(|| format!("read SKILL.md: {}", skill_path.display()))?;
    let folder_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unnamed-skill".to_string());
    let name = frontmatter_value(&content, "name").unwrap_or(folder_name);
    let mut tags = detected_tags(&content);
    if let Some(description) = frontmatter_value(&content, "description") {
        tags.extend(detected_tags(&description));
    }
    tags.sort();
    tags.dedup();

    Ok(Some(ScannedSkillAsset {
        id: format!("skill-{}", to_slug(&name)),
        name,
        asset_type: "skill".to_string(),
        source: skill_path.to_string_lossy().into_owned(),
        origin: "Skill directory scan".to_string(),
        tags,
        content,
        status: "synced".to_string(),
        md5: None,
        systems: Vec::new(),
    }))
}

fn frontmatter_value(content: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    content
        .lines()
        .take(40)
        .find_map(|line| line.trim().strip_prefix(&prefix).map(clean_value))
        .filter(|value| !value.is_empty())
}

fn clean_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

fn detected_tags(content: &str) -> Vec<String> {
    let lower = content.to_ascii_lowercase();
    BUILT_IN_TAGS
        .iter()
        .filter(|tag| lower.contains(**tag))
        .map(|tag| (*tag).to_string())
        .collect()
}

fn skill_root() -> anyhow::Result<PathBuf> {
    Ok(home_dir()?.join(".agents").join("skills"))
}

fn home_dir() -> anyhow::Result<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("cannot resolve user home"))
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::scan_skill_assets_at;

    #[test]
    fn scans_skill_directories_into_sorted_assets() {
        let root = env::temp_dir().join(format!(
            "asset-hub-skills-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let skills_root = root.join(".agents/skills");
        fs::create_dir_all(skills_root.join("zeta")).unwrap();
        fs::create_dir_all(skills_root.join("alpha")).unwrap();
        fs::write(
            skills_root.join("zeta/SKILL.md"),
            "---\nname: Zeta Skill\ndescription: docker helper\n---\n",
        )
        .unwrap();
        fs::write(
            skills_root.join("alpha/SKILL.md"),
            "---\nname: Alpha Skill\ndescription: rust helper\n---\n",
        )
        .unwrap();

        let scanned = scan_skill_assets_at(&skills_root).unwrap();

        assert_eq!(scanned.len(), 2);
        assert_eq!(scanned[0].name, "Alpha Skill");
        assert!(scanned[0].tags.contains(&"rust".to_string()));
        fs::remove_dir_all(root).unwrap();
    }
}
