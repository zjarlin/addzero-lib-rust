use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;

use addzero_plugin_contract::{
    MarkdownSchema, PageSchema, PluginDescriptor, PluginKind, PluginMenuContribution,
    PluginPackageManifest, PluginPage, RuntimeBinding,
};
use addzero_plugin_runtime::create_package_from_dir;
use chrono::Utc;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use super::PluginDescriptorDto;

const CLI_HUB_PROVIDERS_URL: &str = "https://clihub.cc/api/providers";
const CLI_HUB_SCHEMAS_URL: &str = "https://clihub.cc/api/schemas";
const SKILLS_OFFICIAL_URL: &str = "https://skills.sh/official";
const FALLBACK_SKILLS_PAGE_URLS: &[&str] = &[
    "https://skills.sh/microsoft/agent-skills/skill-creator",
    "https://skills.sh/tavily-ai/skills/search",
    "https://skills.sh/openai/openai-agents-python/agents-sdk-quickstart",
];
const MAX_SKILL_SAMPLE_COUNT: usize = 6;
const MAX_SKILL_OWNERS: usize = 3;
const MAX_SKILL_REPOS_PER_OWNER: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketSceneDto {
    Cli,
    Skill,
    Wasm,
}

impl MarketSceneDto {
    pub fn code(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Skill => "skill",
            Self::Wasm => "wasm",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketCatalogItemDto {
    pub id: String,
    pub scene: MarketSceneDto,
    pub source: String,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub description: String,
    pub homepage_url: Option<String>,
    pub repo_url: Option<String>,
    pub install_command: Option<String>,
    pub tags: Vec<String>,
    pub installed: bool,
    pub deploy_label: String,
    pub content: Option<String>,
    pub raw: serde_json::Value,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketSceneSummaryDto {
    pub scene: String,
    pub label: String,
    pub count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketCatalogDto {
    pub generated_at: String,
    pub default_target_dir: String,
    pub scenes: Vec<MarketSceneSummaryDto>,
    pub items: Vec<MarketCatalogItemDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketDeployRequestDto {
    pub target_dir: String,
    pub items: Vec<MarketCatalogItemDto>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketDeployFileDto {
    pub path: String,
    pub kind: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketDeployResultDto {
    pub target_dir: String,
    pub item_count: usize,
    pub files: Vec<MarketDeployFileDto>,
}

#[derive(Clone, Debug, Deserialize)]
struct CliHubProvider {
    name: String,
    display_name: String,
    description: String,
    homepage: String,
    #[serde(default)]
    categories: Vec<String>,
}

pub async fn build_market_catalog(
    installed_skills: BTreeSet<String>,
    wasm_plugins: Vec<PluginDescriptorDto>,
) -> Result<MarketCatalogDto, String> {
    let mut items = Vec::new();
    let client = build_http_client()?;

    match fetch_cli_samples(&client).await {
        Ok(samples) => items.extend(samples),
        Err(err) => items.push(source_error_item(
            MarketSceneDto::Cli,
            "CLI 市场抓取失败",
            "cli-hub",
            err,
        )),
    }
    match fetch_skill_samples(&client, installed_skills).await {
        Ok(samples) => items.extend(samples),
        Err(err) => items.push(source_error_item(
            MarketSceneDto::Skill,
            "Skill 市场抓取失败",
            "skills.sh",
            err,
        )),
    }
    items.extend(build_wasm_samples(wasm_plugins));

    let scenes = [
        (MarketSceneDto::Cli, "CLI 插件市场"),
        (MarketSceneDto::Skill, "Skill 技能市场"),
        (MarketSceneDto::Wasm, "WASM 插件市场"),
    ]
    .into_iter()
    .map(|(scene, label)| MarketSceneSummaryDto {
        scene: scene.code().to_string(),
        label: label.to_string(),
        count: items.iter().filter(|item| item.scene == scene).count(),
    })
    .collect();

    Ok(MarketCatalogDto {
        generated_at: Utc::now().to_rfc3339(),
        default_target_dir: default_bundle_target_dir(),
        scenes,
        items,
    })
}

pub fn deploy_market_bundle(
    input: MarketDeployRequestDto,
) -> Result<MarketDeployResultDto, String> {
    let target_dir = normalize_target_dir(&input.target_dir)?;
    std::fs::create_dir_all(&target_dir)
        .map_err(|err| format!("create target dir {}: {err}", target_dir.display()))?;

    let mut files = Vec::new();
    for item in input.items {
        let item_dir = target_dir
            .join(item.scene.code())
            .join(sanitize_path_component(&item.slug));
        std::fs::create_dir_all(&item_dir)
            .map_err(|err| format!("create item dir {}: {err}", item_dir.display()))?;

        match item.scene {
            MarketSceneDto::Cli => write_cli_bundle(&item_dir, &item, &mut files)?,
            MarketSceneDto::Skill => write_skill_bundle(&item_dir, &item, &mut files)?,
            MarketSceneDto::Wasm => write_wasm_bundle(&item_dir, &item, &mut files)?,
        }
    }

    Ok(MarketDeployResultDto {
        target_dir: target_dir.display().to_string(),
        item_count: files
            .iter()
            .map(|file| parent_bundle_key(&file.path))
            .collect::<BTreeSet<_>>()
            .len(),
        files,
    })
}

fn build_http_client() -> Result<Client, String> {
    Client::builder()
        .user_agent("addzero-aio-market/0.1")
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|err| format!("build market http client: {err}"))
}

async fn fetch_cli_samples(client: &Client) -> Result<Vec<MarketCatalogItemDto>, String> {
    let providers = client
        .get(CLI_HUB_PROVIDERS_URL)
        .send()
        .await
        .map_err(|err| format!("fetch cli-hub providers: {err}"))?
        .json::<Vec<CliHubProvider>>()
        .await
        .map_err(|err| format!("decode cli-hub providers: {err}"))?;

    let schema_value = client
        .get(CLI_HUB_SCHEMAS_URL)
        .send()
        .await
        .map_err(|err| format!("fetch cli-hub schemas: {err}"))?
        .json::<serde_json::Value>()
        .await
        .map_err(|err| format!("decode cli-hub schemas: {err}"))?;

    Ok(providers
        .into_iter()
        .take(8)
        .map(|provider| {
            let sample_command = first_cli_example(&schema_value, &provider.name);
            let summary = if provider.categories.is_empty() {
                "来自 cli-hub 的 CLI provider 样本".to_string()
            } else {
                format!("分类: {}", provider.categories.join(", "))
            };
            MarketCatalogItemDto {
                id: format!("cli:{}", provider.name),
                scene: MarketSceneDto::Cli,
                source: "cli-hub".to_string(),
                slug: provider.name.clone(),
                title: provider.display_name.clone(),
                summary,
                description: provider.description.clone(),
                homepage_url: Some(provider.homepage.clone()),
                repo_url: Some(provider.homepage),
                install_command: sample_command.clone(),
                tags: provider.categories,
                installed: false,
                deploy_label: "导出 CLI provider 清单".to_string(),
                content: sample_command
                    .as_ref()
                    .map(|command| format!("# Sample Command\n\n```bash\n{command}\n```")),
                raw: json!({
                    "provider": provider.name,
                    "sample_command": sample_command,
                }),
            }
        })
        .collect())
}

async fn fetch_skill_samples(
    client: &Client,
    installed_skills: BTreeSet<String>,
) -> Result<Vec<MarketCatalogItemDto>, String> {
    let page_urls = fetch_skill_page_urls(client).await.unwrap_or_else(|_| {
        FALLBACK_SKILLS_PAGE_URLS
            .iter()
            .map(|url| (*url).to_string())
            .collect()
    });
    let mut out = Vec::new();
    for page_url in page_urls.into_iter().take(MAX_SKILL_SAMPLE_COUNT) {
        let html = client
            .get(&page_url)
            .send()
            .await
            .map_err(|err| format!("fetch skills page {page_url}: {err}"))?
            .text()
            .await
            .map_err(|err| format!("read skills page {page_url}: {err}"))?;

        let title =
            capture_first(&html, r#"<h1[^>]*>([^<]+)</h1>"#).unwrap_or_else(|| "skill".to_string());
        let install_command = capture_first(&html, r#"(npx skills add [^<]+)"#);
        let summary = capture_first(&html, r#"<meta name="description" content="([^"]+)""#)
            .unwrap_or_else(|| "来自 skills.sh 的技能页样本".to_string());
        let content =
            extract_skill_snapshot_markdown(&html, &title, &page_url, install_command.as_deref());
        let slug = page_url.rsplit('/').next().unwrap_or("skill").to_string();
        let repo_slug = page_url
            .trim_end_matches('/')
            .split('/')
            .rev()
            .nth(1)
            .unwrap_or("repo")
            .to_string();

        out.push(MarketCatalogItemDto {
            id: format!("skill:{slug}"),
            scene: MarketSceneDto::Skill,
            source: "skills.sh".to_string(),
            slug: slug.clone(),
            title,
            summary,
            description: format!("skills.sh 页面快照，repo: {repo_slug}"),
            homepage_url: Some(page_url.to_string()),
            repo_url: install_command
                .as_deref()
                .and_then(extract_repo_from_install_command),
            install_command,
            tags: vec![repo_slug, "skill".to_string()],
            installed: installed_skills.contains(&slug),
            deploy_label: "导出 skill bundle".to_string(),
            content: Some(content),
            raw: json!({
                "page_url": page_url,
            }),
        });
    }
    Ok(out)
}

async fn fetch_skill_page_urls(client: &Client) -> Result<Vec<String>, String> {
    let html = client
        .get(SKILLS_OFFICIAL_URL)
        .send()
        .await
        .map_err(|err| format!("fetch skills official page: {err}"))?
        .text()
        .await
        .map_err(|err| format!("read skills official page: {err}"))?;

    let owners = extract_skill_owners(&html);
    if owners.is_empty() {
        return Err("no owners found on skills.sh/official".to_string());
    }

    let mut urls = Vec::new();
    let mut seen = BTreeSet::new();
    for owner in owners.into_iter().take(MAX_SKILL_OWNERS) {
        let owner_urls = match fetch_skill_page_urls_for_owner(client, &owner).await {
            Ok(owner_urls) => owner_urls,
            Err(_) => continue,
        };
        for url in owner_urls {
            if seen.insert(url.clone()) {
                urls.push(url);
                if urls.len() >= MAX_SKILL_SAMPLE_COUNT {
                    return Ok(urls);
                }
            }
        }
    }

    if urls.is_empty() {
        return Err("no skill detail urls found from official listings".to_string());
    }

    Ok(urls)
}

async fn fetch_skill_page_urls_for_owner(
    client: &Client,
    owner: &str,
) -> Result<Vec<String>, String> {
    let owner_url = format!("https://skills.sh/{owner}");
    let html = client
        .get(&owner_url)
        .send()
        .await
        .map_err(|err| format!("fetch skills owner page {owner_url}: {err}"))?
        .text()
        .await
        .map_err(|err| format!("read skills owner page {owner_url}: {err}"))?;

    let row_regex = Regex::new(
        r#"(?s)href="/(?P<owner>[^"/]+)/(?P<repo>[^"/]+)".*?skills<!-- -->:<!-- --> <!-- -->(?P<skill>[^,<]+)"#,
    )
    .map_err(|err| format!("compile skill owner regex: {err}"))?;

    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for captures in row_regex.captures_iter(&html) {
        let row_owner = captures
            .name("owner")
            .map(|value| value.as_str())
            .unwrap_or_default();
        if row_owner != owner {
            continue;
        }
        let repo = captures
            .name("repo")
            .map(|value| value.as_str().trim())
            .unwrap_or_default();
        let skill = captures
            .name("skill")
            .map(|value| value.as_str().trim())
            .unwrap_or_default();
        if repo.is_empty() || skill.is_empty() {
            continue;
        }
        let url = format!("https://skills.sh/{owner}/{repo}/{skill}");
        if seen.insert(url.clone()) {
            out.push(url);
        }
        if out.len() >= MAX_SKILL_REPOS_PER_OWNER {
            break;
        }
    }

    if out.is_empty() {
        return Err(format!("no repo skill rows found for owner {owner}"));
    }

    Ok(out)
}

fn build_wasm_samples(wasm_plugins: Vec<PluginDescriptorDto>) -> Vec<MarketCatalogItemDto> {
    wasm_plugins
        .into_iter()
        .map(|plugin| MarketCatalogItemDto {
            id: format!("wasm:{}", plugin.manifest_id),
            scene: MarketSceneDto::Wasm,
            source: if plugin.builtin {
                "aio-runtime builtin".to_string()
            } else {
                "aio-runtime".to_string()
            },
            slug: plugin.manifest_id.clone(),
            title: plugin.name.clone(),
            summary: format!("状态: {} · 扩展点: {}", plugin.state, plugin.extension_points.join(", ")),
            description: plugin.description.clone(),
            homepage_url: None,
            repo_url: None,
            install_command: None,
            tags: plugin.extension_points.clone(),
            installed: true,
            deploy_label: "导出 wasm 插件清单".to_string(),
            content: Some(format!(
                "# {}\n\n- manifest_id: `{}`\n- version: `{}`\n- state: `{}`\n- builtin: `{}`\n- permissions: {}\n",
                plugin.name,
                plugin.manifest_id,
                plugin.version,
                plugin.state,
                plugin.builtin,
                if plugin.permissions.is_empty() {
                    "(none)".to_string()
                } else {
                    plugin.permissions.join(", ")
                }
            )),
            raw: json!(plugin),
        })
        .collect()
}

fn source_error_item(
    scene: MarketSceneDto,
    title: &str,
    source: &str,
    error: String,
) -> MarketCatalogItemDto {
    MarketCatalogItemDto {
        id: format!("{}:source-error", scene.code()),
        scene,
        source: source.to_string(),
        slug: "source-error".to_string(),
        title: title.to_string(),
        summary: "远端源暂时不可用，但当前场景仍保留本地工作流。".to_string(),
        description: error.clone(),
        homepage_url: None,
        repo_url: None,
        install_command: None,
        tags: vec!["error".to_string()],
        installed: false,
        deploy_label: "等待源恢复".to_string(),
        content: Some(error.clone()),
        raw: json!({ "error": error }),
    }
}

fn write_cli_bundle(
    item_dir: &Path,
    item: &MarketCatalogItemDto,
    files: &mut Vec<MarketDeployFileDto>,
) -> Result<(), String> {
    write_market_plugin_scaffold(item_dir, item, files)
}

fn write_skill_bundle(
    item_dir: &Path,
    item: &MarketCatalogItemDto,
    files: &mut Vec<MarketDeployFileDto>,
) -> Result<(), String> {
    write_market_plugin_scaffold(item_dir, item, files)
}

fn write_wasm_bundle(
    item_dir: &Path,
    item: &MarketCatalogItemDto,
    files: &mut Vec<MarketDeployFileDto>,
) -> Result<(), String> {
    write_market_plugin_scaffold(item_dir, item, files)
}

fn write_market_plugin_scaffold(
    item_dir: &Path,
    item: &MarketCatalogItemDto,
    files: &mut Vec<MarketDeployFileDto>,
) -> Result<(), String> {
    let backend_dir = item_dir.join("backend");
    let assets_dir = item_dir.join("assets");
    std::fs::create_dir_all(&backend_dir)
        .map_err(|err| format!("create backend dir {}: {err}", backend_dir.display()))?;
    std::fs::create_dir_all(&assets_dir)
        .map_err(|err| format!("create assets dir {}: {err}", assets_dir.display()))?;

    let plugin_manifest = build_plugin_package_manifest(item);
    let plugin_toml = toml_edit::ser::to_string_pretty(&plugin_manifest)
        .map_err(|err| format!("serialize plugin.toml: {err}"))?;
    write_text_file(
        &item_dir.join("plugin.toml"),
        plugin_toml,
        "plugin-manifest",
        files,
    )?;

    write_bytes_file(
        &backend_dir.join("plugin.wasm"),
        minimal_wasm_module_bytes(),
        "wasm-module",
        files,
    )?;

    write_json_file(&assets_dir.join("market-item.json"), item, files)?;
    write_text_file(
        &assets_dir.join("README.md"),
        render_plugin_readme(item),
        "asset-readme",
        files,
    )?;
    if let Some(command) = &item.install_command {
        write_text_file(
            &assets_dir.join("install.sh"),
            format!("#!/usr/bin/env bash\n{command}\n"),
            "install-script",
            files,
        )?;
    }
    if let Some(content) = &item.content {
        write_text_file(
            &assets_dir.join("snapshot.md"),
            content.clone(),
            "snapshot",
            files,
        )?;
    }

    let checksum_entries = collect_checksum_entries(item_dir)?;
    let checksums = checksum_entries
        .iter()
        .map(|(relative, bytes)| format!("{}  {relative}", sha256_hex(bytes)))
        .collect::<Vec<_>>()
        .join("\n");
    write_text_file(
        &item_dir.join("checksums.sha256"),
        format!("{checksums}\n"),
        "checksums",
        files,
    )?;

    let package_name = format!("{}.azplugin", plugin_package_dir_name(item));
    let package_path = item_dir.join(&package_name);
    if package_path.exists() {
        std::fs::remove_file(&package_path)
            .map_err(|err| format!("remove old package {}: {err}", package_path.display()))?;
    }
    let temp_package_path = env::temp_dir().join(format!(
        "aio-market-{}-{}.azplugin",
        plugin_package_dir_name(item),
        Utc::now().timestamp_millis()
    ));
    create_package_from_dir(item_dir, &temp_package_path).map_err(|err| {
        format!(
            "create plugin package {}: {err}",
            temp_package_path.display()
        )
    })?;
    std::fs::rename(&temp_package_path, &package_path).map_err(|err| {
        format!(
            "move plugin package {} -> {}: {err}",
            temp_package_path.display(),
            package_path.display()
        )
    })?;
    files.push(MarketDeployFileDto {
        path: package_path.display().to_string(),
        kind: "azplugin".to_string(),
    });
    Ok(())
}

fn write_json_file<T: Serialize>(
    path: &Path,
    value: &T,
    files: &mut Vec<MarketDeployFileDto>,
) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|err| format!("serialize json: {err}"))?;
    std::fs::write(path, bytes).map_err(|err| format!("write {}: {err}", path.display()))?;
    files.push(MarketDeployFileDto {
        path: path.display().to_string(),
        kind: "json".to_string(),
    });
    Ok(())
}

fn write_text_file(
    path: &Path,
    text: String,
    kind: &str,
    files: &mut Vec<MarketDeployFileDto>,
) -> Result<(), String> {
    std::fs::write(path, text).map_err(|err| format!("write {}: {err}", path.display()))?;
    files.push(MarketDeployFileDto {
        path: path.display().to_string(),
        kind: kind.to_string(),
    });
    Ok(())
}

fn write_bytes_file(
    path: &Path,
    bytes: Vec<u8>,
    kind: &str,
    files: &mut Vec<MarketDeployFileDto>,
) -> Result<(), String> {
    std::fs::write(path, bytes).map_err(|err| format!("write {}: {err}", path.display()))?;
    files.push(MarketDeployFileDto {
        path: path.display().to_string(),
        kind: kind.to_string(),
    });
    Ok(())
}

fn render_plugin_readme(item: &MarketCatalogItemDto) -> String {
    let mut lines = vec![
        format!("# {}", item.title),
        String::new(),
        "该目录是从市场对象导出的 `.azplugin` 插件脚手架。".to_string(),
        String::new(),
        format!("- scene: `{}`", item.scene.code()),
        format!("- source: `{}`", item.source),
        format!("- slug: `{}`", item.slug),
        format!("- plugin_id: `{}`", plugin_id_for_market_item(item)),
        format!("- entry: `backend/plugin.wasm`"),
    ];
    if let Some(url) = &item.homepage_url {
        lines.push(format!("- homepage: {url}"));
    }
    if let Some(url) = &item.repo_url {
        lines.push(format!("- repo: {url}"));
    }
    if let Some(command) = &item.install_command {
        lines.push(format!("- install: `{command}`"));
    }
    lines.push(String::new());
    lines.push(item.summary.clone());
    lines.push(String::new());
    lines.push(item.description.clone());
    lines.push(String::new());
    lines.join("\n")
}

fn build_plugin_package_manifest(item: &MarketCatalogItemDto) -> PluginPackageManifest {
    PluginPackageManifest {
        descriptor: PluginDescriptor {
            id: plugin_id_for_market_item(item),
            name: item.title.clone(),
            version: "0.1.0".to_string(),
            kind: PluginKind::Business,
            summary: item.summary.clone(),
            tags: plugin_tags_for_market_item(item),
            icon: None,
            compatibility: vec!["aio".to_string(), "desktop".to_string(), "web".to_string()],
            capabilities: vec![],
            menus: vec![PluginMenuContribution {
                section: "市场导入插件".to_string(),
                label: item.title.clone(),
                page_id: "overview".to_string(),
                order: 100,
                icon: None,
            }],
            pages: vec![PluginPage {
                id: "overview".to_string(),
                title: item.title.clone(),
                subtitle: format!("从 {} 导入的 {} 市场对象", item.source, item.scene.code()),
                schema: PageSchema::Markdown(MarkdownSchema {
                    body: render_plugin_markdown_page(item),
                }),
            }],
        },
        runtime: RuntimeBinding {
            binary_path: "backend/plugin.wasm".to_string(),
            checksum_path: "checksums.sha256".to_string(),
            assets_dir: Some("assets".to_string()),
        },
        default_instance_label: Some(item.title.clone()),
    }
}

fn render_plugin_markdown_page(item: &MarketCatalogItemDto) -> String {
    let mut lines = vec![
        format!("# {}", item.title),
        String::new(),
        format!("- source: {}", item.source),
        format!("- scene: {}", item.scene.code()),
        format!("- slug: {}", item.slug),
        String::new(),
        item.summary.clone(),
        String::new(),
        item.description.clone(),
    ];
    if let Some(command) = &item.install_command {
        lines.push(String::new());
        lines.push("## Install Hint".to_string());
        lines.push(String::new());
        lines.push(format!("```bash\n{command}\n```"));
    }
    if let Some(content) = &item.content {
        lines.push(String::new());
        lines.push("## Snapshot".to_string());
        lines.push(String::new());
        lines.push(content.clone());
    }
    lines.join("\n")
}

fn plugin_id_for_market_item(item: &MarketCatalogItemDto) -> String {
    format!(
        "market.{}.{}",
        item.scene.code(),
        sanitize_path_component(&item.slug).replace('-', "_")
    )
}

fn plugin_package_dir_name(item: &MarketCatalogItemDto) -> String {
    format!(
        "{}-{}",
        item.scene.code(),
        sanitize_path_component(&item.slug)
    )
}

fn plugin_tags_for_market_item(item: &MarketCatalogItemDto) -> Vec<String> {
    let mut tags = vec![
        "market-import".to_string(),
        item.scene.code().to_string(),
        sanitize_path_component(&item.source),
    ];
    tags.extend(item.tags.iter().map(|tag| sanitize_path_component(tag)));
    tags.into_iter()
        .filter(|tag| !tag.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn minimal_wasm_module_bytes() -> Vec<u8> {
    vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]
}

fn collect_checksum_entries(item_dir: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut entries = Vec::new();
    collect_checksum_entries_recursive(item_dir, item_dir, &mut entries)?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(entries)
}

fn collect_checksum_entries_recursive(
    root: &Path,
    dir: &Path,
    entries: &mut Vec<(String, Vec<u8>)>,
) -> Result<(), String> {
    for entry in
        std::fs::read_dir(dir).map_err(|err| format!("read dir {}: {err}", dir.display()))?
    {
        let entry = entry.map_err(|err| format!("read dir entry {}: {err}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_checksum_entries_recursive(root, &path, entries)?;
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if matches!(file_name, "checksums.sha256") || file_name.ends_with(".azplugin") {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|err| format!("strip prefix {}: {err}", path.display()))?
            .to_string_lossy()
            .replace('\\', "/");
        let bytes =
            std::fs::read(&path).map_err(|err| format!("read {}: {err}", path.display()))?;
        entries.push((relative, bytes));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}

fn first_cli_example(value: &serde_json::Value, provider_name: &str) -> Option<String> {
    value
        .get(provider_name)?
        .get("operations")?
        .as_array()?
        .iter()
        .find_map(|operation| {
            operation
                .get("example")
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string)
        })
}

fn extract_skill_snapshot_markdown(
    html: &str,
    title: &str,
    page_url: &str,
    install_command: Option<&str>,
) -> String {
    let summary =
        capture_first(html, r#"<meta name="description" content="([^"]+)""#).unwrap_or_default();
    let body = capture_first(
        html,
        r#"(?s)SKILL\.md</span></div><div class="prose[^"]*">(.*?)</div></div></div>"#,
    )
    .map(|block| strip_html(&block))
    .unwrap_or_default();

    let mut out = vec![
        format!("# {title}"),
        String::new(),
        format!("Source: {page_url}"),
    ];
    if let Some(command) = install_command {
        out.push(format!("Install: `{command}`"));
    }
    if !summary.is_empty() {
        out.push(String::new());
        out.push("## Summary".to_string());
        out.push(String::new());
        out.push(summary);
    }
    if !body.is_empty() {
        out.push(String::new());
        out.push("## Snapshot".to_string());
        out.push(String::new());
        out.push(body.lines().take(60).collect::<Vec<_>>().join("\n"));
    }
    out.join("\n")
}

fn extract_repo_from_install_command(command: &str) -> Option<String> {
    let repo = Regex::new(r#"https://github\.com/[^\s]+/[^\s]+"#).ok()?;
    repo.find(command).map(|value| value.as_str().to_string())
}

fn extract_skill_owners(html: &str) -> Vec<String> {
    let regex = match Regex::new(r#"href="/([a-zA-Z0-9][a-zA-Z0-9-]*)""#) {
        Ok(regex) => regex,
        Err(_) => return Vec::new(),
    };

    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for captures in regex.captures_iter(html) {
        let owner = captures
            .get(1)
            .map(|value| value.as_str())
            .unwrap_or_default();
        if !is_valid_skill_owner_slug(owner) {
            continue;
        }
        if seen.insert(owner.to_string()) {
            out.push(owner.to_string());
        }
    }
    out
}

fn is_valid_skill_owner_slug(value: &str) -> bool {
    !matches!(value, "official" | "audits" | "docs" | "api")
        && value != "favicon.ico"
        && !value.starts_with('_')
}

fn capture_first(content: &str, pattern: &str) -> Option<String> {
    let regex = Regex::new(pattern).ok()?;
    let captures = regex.captures(content)?;
    let value = captures.get(1)?.as_str();
    Some(html_decode(value.trim()))
}

fn strip_html(value: &str) -> String {
    let tags = Regex::new(r"<[^>]+>").ok();
    let without_tags = tags
        .as_ref()
        .map(|regex| regex.replace_all(value, "\n").to_string())
        .unwrap_or_else(|| value.to_string());
    let whitespace = Regex::new(r"\n{3,}").ok();
    let collapsed = whitespace
        .as_ref()
        .map(|regex| regex.replace_all(&without_tags, "\n\n").to_string())
        .unwrap_or(without_tags);
    html_decode(collapsed.trim())
}

fn html_decode(value: impl AsRef<str>) -> String {
    value
        .as_ref()
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#x3C;", "<")
        .replace("&#x3E;", ">")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn normalize_target_dir(value: &str) -> Result<PathBuf, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("target_dir is required".to_string());
    }
    Ok(PathBuf::from(trimmed))
}

fn parent_bundle_key(path: &str) -> String {
    Path::new(path)
        .parent()
        .map(|parent| parent.display().to_string())
        .unwrap_or_else(|| path.to_string())
}

fn sanitize_path_component(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

pub fn default_bundle_target_dir() -> String {
    let base = env::var("HOME")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join(".addzero")
        .join("aio-market-bundles")
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_path_component_keeps_safe_chars() {
        assert_eq!(sanitize_path_component("com.addzero/ui"), "com.addzero-ui");
    }

    #[test]
    fn extract_repo_from_command_works() {
        let repo = extract_repo_from_install_command(
            "npx skills add https://github.com/tavily-ai/skills --skill search",
        );
        assert_eq!(repo.as_deref(), Some("https://github.com/tavily-ai/skills"));
    }

    #[test]
    fn extract_skill_owners_skips_navigation_entries() {
        let html = r#"
        <a href="/official">Official</a>
        <a href="/microsoft">Microsoft</a>
        <a href="/tavily-ai">Tavily</a>
        "#;
        assert_eq!(
            extract_skill_owners(html),
            vec!["microsoft".to_string(), "tavily-ai".to_string()]
        );
    }
}
