#![forbid(unsafe_code)]

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use az_aio_plugin_api::api::{
    BackendApiContribution, CatalogItemContribution, CatalogItemKind, CatalogProviderContribution,
    CatalogSource, CatalogTagContribution, CatalogTagGroup, ContributionSet, NativeAzAioPlugin,
    NativePluginContext, NativePluginRuntime, PluginActivation, PluginDescriptor, PluginKind,
    UiContribution, UiContributionSlot,
};
use az_aio_plugin_api::register_native_plugin;

const SYSTEM_SKILLS_ROOT: &str = "/Users/zjarlin/.codex/skills/.system";
const USER_SKILLS_ROOT: &str = "/Users/zjarlin/.agents/skills";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillRoot {
    pub path: PathBuf,
    pub section: String,
    pub source: CatalogSource,
}

impl SkillRoot {
    pub fn new(
        path: impl Into<PathBuf>,
        section: impl Into<String>,
        source: CatalogSource,
    ) -> Self {
        Self {
            path: path.into(),
            section: section.into(),
            source,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GitSkillsPlugin {
    roots: Vec<SkillRoot>,
}

impl GitSkillsPlugin {
    pub fn new(roots: Vec<SkillRoot>) -> Self {
        Self { roots }
    }
}

impl Default for GitSkillsPlugin {
    fn default() -> Self {
        Self {
            roots: vec![
                SkillRoot::new(SYSTEM_SKILLS_ROOT, "系统", CatalogSource::System),
                SkillRoot::new(USER_SKILLS_ROOT, "用户技能", CatalogSource::User),
            ],
        }
    }
}

impl NativeAzAioPlugin for GitSkillsPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            id: "git/skills".to_string(),
            name: "Git 技能".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "扫描 Codex 系统技能和用户技能，支持软链接。".to_string(),
            activation: PluginActivation::Eager,
            priority: 800,
            dependencies: Vec::new(),
            capabilities: vec!["catalog-provider".to_string(), "skill-scan".to_string()],
            permissions: self
                .roots
                .iter()
                .map(|root| format!("读取 {}", root.path.display()))
                .collect(),
            kind: PluginKind::Native,
        }
    }

    fn contributions(&self) -> anyhow::Result<ContributionSet> {
        Ok(ContributionSet {
            nav_items: Vec::new(),
            pages: Vec::new(),
            toolbar_actions: Vec::new(),
            catalog_providers: vec![scan_skill_roots(&self.roots)],
            ui_contributions: vec![ui_contribution(
                "git.skills.ui.catalog",
                UiContributionSlot::Content,
                "技能目录贡献",
                "git.skills.catalog-provider",
                Some("/plugins"),
                20,
            )],
            backend_apis: skill_backend_apis(),
            settings_sections: Vec::new(),
            shell_entries: Vec::new(),
            generated_files: Vec::new(),
        })
    }

    fn runtime(&self, _context: NativePluginContext) -> anyhow::Result<NativePluginRuntime> {
        Ok(NativePluginRuntime::default())
    }
}

register_native_plugin!(GitSkillsPlugin);

pub fn ensure_linked() {}

fn skill_backend_apis() -> Vec<BackendApiContribution> {
    vec![backend_api(
        "git.skills.api.scan",
        "GET",
        "/api/git/skills",
        "扫描技能",
        "返回系统技能和用户技能目录的 SKILL.md 元数据。",
        10,
    )]
}

fn ui_contribution(
    id: &str,
    slot: UiContributionSlot,
    label: &str,
    renderer_id: &str,
    route: Option<&str>,
    order: i32,
) -> UiContribution {
    UiContribution {
        id: id.to_string(),
        slot,
        label: label.to_string(),
        renderer_id: renderer_id.to_string(),
        route: route.map(str::to_string),
        order,
    }
}

fn backend_api(
    id: &str,
    method: &str,
    path: &str,
    label: &str,
    description: &str,
    order: i32,
) -> BackendApiContribution {
    BackendApiContribution {
        id: id.to_string(),
        method: method.to_string(),
        path: path.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        order,
    }
}

pub fn scan_skill_roots(roots: &[SkillRoot]) -> CatalogProviderContribution {
    let mut seen_skill_files = HashSet::new();
    let mut items = Vec::new();

    for root in roots {
        let mut entries = skill_root_entries(&root.path);
        entries.sort();

        for skill_dir in entries {
            if let Some(item) = load_skill_catalog_item(root, &skill_dir, &mut seen_skill_files) {
                items.push(item);
            }
        }
    }

    items.sort_by(|left, right| {
        left.section
            .cmp(&right.section)
            .then(left.name.cmp(&right.name))
            .then(left.id.cmp(&right.id))
    });

    CatalogProviderContribution {
        id: "skills.local-roots".to_string(),
        label: "技能".to_string(),
        order: 20,
        items,
    }
}

fn skill_root_entries(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            !is_hidden_path(path)
                && fs::metadata(path)
                    .map(|metadata| metadata.is_dir())
                    .unwrap_or(false)
        })
        .collect()
}

fn is_hidden_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}

fn load_skill_catalog_item(
    root: &SkillRoot,
    skill_dir: &Path,
    seen_skill_files: &mut HashSet<PathBuf>,
) -> Option<CatalogItemContribution> {
    let skill_file = skill_dir.join("SKILL.md");
    if !fs::metadata(&skill_file)
        .map(|metadata| metadata.is_file())
        .unwrap_or(false)
    {
        return None;
    }

    let canonical_skill_file = skill_file
        .canonicalize()
        .unwrap_or_else(|_| skill_file.clone());
    if !seen_skill_files.insert(canonical_skill_file.clone()) {
        return None;
    }

    let content = fs::read_to_string(&skill_file).ok()?;
    let fallback_name = skill_dir
        .file_name()
        .and_then(|name| name.to_str())
        .map(skill_name_from_dir)
        .unwrap_or_else(|| "未命名技能".to_string());
    let name = front_matter_value(&content, "name").unwrap_or(fallback_name);
    let description = front_matter_value(&content, "description")
        .unwrap_or_else(|| "SKILL.md 未提供描述".to_string());
    let tags = classify_skill_tags(&name, &description, &canonical_skill_file, &content);

    Some(CatalogItemContribution {
        id: format!("skill.{}", canonical_skill_file.display()),
        name,
        description,
        section: root.section.clone(),
        icon: skill_icon(root.source).to_string(),
        accent_class: skill_accent_class(root.source).to_string(),
        kind: CatalogItemKind::Skill,
        source: root.source,
        installed: true,
        tags,
        permissions: vec![
            format!("读取 {}", root.path.display()),
            "解析 SKILL.md 元数据".to_string(),
            "跟随软链接并用真实路径去重".to_string(),
        ],
        path: Some(canonical_skill_file.display().to_string()),
    })
}

fn classify_skill_tags(
    name: &str,
    description: &str,
    skill_file: &Path,
    content: &str,
) -> Vec<CatalogTagContribution> {
    let haystack = format!(
        "{}\n{}\n{}\n{}",
        name,
        description,
        skill_file.display(),
        content
    )
    .to_lowercase();
    let mut tags = Vec::new();

    // Existing skills are already useful without a metadata migration, so tags are
    // derived from stable names, paths, descriptions, and common ecosystem terms.
    for (id, label, keywords) in [
        (
            "dev.rust",
            "Rust",
            &["rust", "cargo", "clippy", "rustc", "wasm", "dioxus"][..],
        ),
        ("dev.java", "Java", &["java", "jvm", "spring", "ktor"][..]),
        ("dev.gradle", "Gradle", &["gradle", "gradlew"][..]),
        ("dev.maven", "Maven", &["maven", "pom.xml", "mvn"][..]),
        (
            "dev.cmp",
            "CMP",
            &["cmp", "compose multiplatform", "compose desktop"][..],
        ),
        (
            "dev.kmp",
            "KMP",
            &["kmp", "kotlin multiplatform", "commonmain", "kotlin"][..],
        ),
        (
            "dev.convention",
            "编程规范",
            &[
                "convention",
                "coding",
                "best-practices",
                "rules",
                "规范",
                "约定",
                "lint",
            ][..],
        ),
    ] {
        if keywords.iter().any(|keyword| haystack.contains(keyword)) {
            tags.push(skill_tag(id, label, CatalogTagGroup::Developer));
        }
    }

    if [
        "design",
        "frontend",
        "ui",
        "ux",
        "a11y",
        "accessibility",
        "brand",
        "stitch",
        "seo",
        "设计",
        "视觉",
    ]
    .iter()
    .any(|keyword| haystack.contains(keyword))
    {
        tags.push(skill_tag("design", "设计", CatalogTagGroup::Design));
    }

    tags
}

fn skill_tag(id: &str, label: &str, group: CatalogTagGroup) -> CatalogTagContribution {
    CatalogTagContribution {
        id: id.to_string(),
        label: label.to_string(),
        group,
    }
}

fn skill_icon(source: CatalogSource) -> &'static str {
    match source {
        CatalogSource::System => "◆",
        CatalogSource::User => "◩",
        _ => "◇",
    }
}

fn skill_accent_class(source: CatalogSource) -> &'static str {
    match source {
        CatalogSource::System => "plugin-icon--system-skill",
        CatalogSource::User => "plugin-icon--user-skill",
        _ => "plugin-icon--git",
    }
}

fn skill_name_from_dir(name: &str) -> String {
    name.split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn front_matter_value(content: &str, key: &str) -> Option<String> {
    let lines = content.lines().collect::<Vec<_>>();
    if lines.first().copied() != Some("---") {
        return None;
    }

    let key_prefix = format!("{key}:");
    let mut index = 1;
    while index < lines.len() {
        let line = lines[index];
        if line.trim() == "---" {
            break;
        }

        if let Some(value) = line.strip_prefix(&key_prefix) {
            let value = value.trim();
            if value == ">" || value == "|" {
                return folded_front_matter_value(&lines, index + 1);
            }

            let value = value.trim_matches('"').trim_matches('\'').trim();
            return (!value.is_empty()).then(|| value.to_string());
        }

        index += 1;
    }

    None
}

fn folded_front_matter_value(lines: &[&str], start_index: usize) -> Option<String> {
    let mut parts = Vec::new();

    for line in lines.iter().skip(start_index) {
        if line.trim() == "---" {
            break;
        }
        if !line.starts_with(' ') && !line.starts_with('\t') && line.contains(':') {
            break;
        }

        let trimmed = line.trim();
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }

    (!parts.is_empty()).then(|| parts.join(" "))
}
