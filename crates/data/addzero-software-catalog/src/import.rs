use std::sync::OnceLock;
use std::time::Duration;

use regex::Regex;
use reqwest::{Client, Url};

use crate::model::{
    InstallerKind, SoftwareCatalogError, SoftwareCatalogResult, SoftwareDraftInput,
    SoftwareEntryInput, SoftwareInstallMethodDto, SoftwareMetadataDto, SoftwareMetadataFetchInput,
    SoftwarePlatform,
};

fn http_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(8))
            .build()
            .expect("failed to build reqwest::Client")
    })
}

fn title_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?is)<title[^>]*>(.*?)</title>").unwrap())
}

fn description_meta_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?is)<meta[^>]+name=["']description["'][^>]+content=["'](.*?)["'][^>]*>"#)
            .unwrap()
    })
}

fn description_meta_reversed_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?is)<meta[^>]+content=["'](.*?)["'][^>]+name=["']description["'][^>]*>"#)
            .unwrap()
    })
}

fn icon_href_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?is)<link[^>]+rel=["'][^"']*icon[^"']*["'][^>]+href=["'](.*?)["'][^>]*>"#)
            .unwrap()
    })
}

pub(crate) async fn fetch_metadata(
    input: SoftwareMetadataFetchInput,
) -> SoftwareCatalogResult<SoftwareMetadataDto> {
    let homepage_url = input.homepage_url.trim().to_string();
    if homepage_url.is_empty() {
        return Err(SoftwareCatalogError::Message(
            "请先填写官网 URL，再执行抓取。".to_string(),
        ));
    }

    let url = Url::parse(&homepage_url)
        .map_err(|err| SoftwareCatalogError::fetch(format!("官网 URL 非法：{err}")))?;
    let client = http_client();
    let response = client
        .get(url.clone())
        .send()
        .await
        .map_err(|err| SoftwareCatalogError::fetch(format!("抓取官网失败：{err}")))?;
    let final_url = response.url().clone();
    let body = response
        .text()
        .await
        .map_err(|err| SoftwareCatalogError::fetch(format!("读取官网响应失败：{err}")))?;

    let title = extract_first(&body, title_re()).unwrap_or_default();
    let summary = extract_first(&body, description_meta_re())
        .or_else(|| extract_first(&body, description_meta_reversed_re()))
        .unwrap_or_default();
    let icon_url =
        extract_icon_url(&body, &final_url).unwrap_or_else(|| default_favicon_url(&final_url));

    Ok(SoftwareMetadataDto {
        title: html_unescape(&title),
        summary: html_unescape(&summary),
        homepage_url: final_url.to_string(),
        icon_url,
    })
}

pub(crate) async fn build_draft(
    input: SoftwareDraftInput,
) -> SoftwareCatalogResult<SoftwareEntryInput> {
    let metadata = fetch_metadata(SoftwareMetadataFetchInput {
        homepage_url: input.homepage_url,
    })
    .await?;
    let slug = infer_slug(&metadata)?;
    let title = normalized_title(&metadata.title, &slug);
    let vendor = infer_vendor(&metadata, &title);
    let tags = infer_tags(&slug, &title, &vendor);
    let methods = build_methods(&slug, &title, &input.preferred_platforms);

    Ok(SoftwareEntryInput {
        id: None,
        slug,
        title,
        vendor,
        summary: metadata.summary,
        homepage_url: metadata.homepage_url,
        icon_url: best_icon_url(&metadata.icon_url),
        trial_platforms: default_platforms(&input.preferred_platforms),
        tags,
        methods,
    })
}

pub(crate) fn seed_entries() -> Vec<SoftwareEntryInput> {
    vec![
        SoftwareEntryInput {
            id: None,
            slug: "cursor".to_string(),
            title: "Cursor".to_string(),
            vendor: "Anysphere".to_string(),
            summary: "AI IDE，预置 macOS / Windows 两个平台的常见安装方式。".to_string(),
            homepage_url: "https://cursor.com".to_string(),
            icon_url: "https://cdn.simpleicons.org/cursor".to_string(),
            trial_platforms: vec![SoftwarePlatform::Macos, SoftwarePlatform::Windows],
            tags: vec!["ide".to_string(), "agent".to_string()],
            methods: vec![
                method(
                    SoftwarePlatform::Macos,
                    InstallerKind::Brew,
                    "brew cask",
                    "cursor",
                    "brew install --cask cursor",
                    "适合统一工作站初始化。",
                ),
                method(
                    SoftwarePlatform::Macos,
                    InstallerKind::DirectPackage,
                    "DMG",
                    "cursor-macos-universal.dmg",
                    "open ~/Downloads/cursor-macos-universal.dmg",
                    "可配合归档安装包一起管理。",
                ),
                method(
                    SoftwarePlatform::Windows,
                    InstallerKind::Winget,
                    "winget",
                    "Anysphere.Cursor",
                    "winget install --id Anysphere.Cursor -e",
                    "Windows 机常见默认入口。",
                ),
            ],
        },
        SoftwareEntryInput {
            id: None,
            slug: "obsidian".to_string(),
            title: "Obsidian".to_string(),
            vendor: "Obsidian".to_string(),
            summary: "笔记工作流软件，支持包管理器和直链下载两类安装方式。".to_string(),
            homepage_url: "https://obsidian.md".to_string(),
            icon_url: "https://cdn.simpleicons.org/obsidian".to_string(),
            trial_platforms: vec![SoftwarePlatform::Macos, SoftwarePlatform::Windows],
            tags: vec!["notes".to_string(), "knowledge".to_string()],
            methods: vec![
                method(
                    SoftwarePlatform::Macos,
                    InstallerKind::Brew,
                    "brew cask",
                    "obsidian",
                    "brew install --cask obsidian",
                    "Mac 侧走 Homebrew 最省事。",
                ),
                method(
                    SoftwarePlatform::Windows,
                    InstallerKind::Winget,
                    "winget",
                    "Obsidian.Obsidian",
                    "winget install --id Obsidian.Obsidian -e",
                    "Windows 侧对齐官方包名。",
                ),
                method(
                    SoftwarePlatform::Windows,
                    InstallerKind::Curl,
                    "curl 直链",
                    "Obsidian-Installer.exe",
                    "curl -L https://github.com/obsidianmd/obsidian-releases/releases/latest/download/Obsidian.1.0.0.exe -o Obsidian-Installer.exe",
                    "给不能用 winget 的环境留后门。",
                ),
            ],
        },
        SoftwareEntryInput {
            id: None,
            slug: "bun".to_string(),
            title: "Bun".to_string(),
            vendor: "Oven".to_string(),
            summary: "运行时和包管理器本身也可以作为软件目录条目维护。".to_string(),
            homepage_url: "https://bun.sh".to_string(),
            icon_url: "https://cdn.simpleicons.org/bun".to_string(),
            trial_platforms: vec![
                SoftwarePlatform::Macos,
                SoftwarePlatform::Windows,
                SoftwarePlatform::Linux,
            ],
            tags: vec!["runtime".to_string(), "package-manager".to_string()],
            methods: vec![
                method(
                    SoftwarePlatform::Macos,
                    InstallerKind::Brew,
                    "brew formula",
                    "bun",
                    "brew install bun",
                    "适合开发机标准化安装。",
                ),
                method(
                    SoftwarePlatform::Windows,
                    InstallerKind::Winget,
                    "winget",
                    "Oven-sh.Bun",
                    "winget install --id Oven-sh.Bun -e",
                    "Windows 侧优先 winget。",
                ),
                method(
                    SoftwarePlatform::Linux,
                    InstallerKind::Curl,
                    "curl 安装脚本",
                    "bun-install",
                    "curl -fsSL https://bun.sh/install | bash",
                    "保留 shell 直装命令。",
                ),
            ],
        },
        SoftwareEntryInput {
            id: None,
            slug: "wechat".to_string(),
            title: "微信".to_string(),
            vendor: "Tencent".to_string(),
            summary: "示例一个默认只在 Windows 试用的平台，但仍然允许在 macOS 管理。".to_string(),
            homepage_url: "https://weixin.qq.com".to_string(),
            icon_url: "https://cdn.simpleicons.org/wechat".to_string(),
            trial_platforms: vec![SoftwarePlatform::Windows],
            tags: vec!["im".to_string(), "office".to_string()],
            methods: vec![
                method(
                    SoftwarePlatform::Windows,
                    InstallerKind::Winget,
                    "winget",
                    "Tencent.WeChat",
                    "winget install --id Tencent.WeChat -e",
                    "官方桌面端。",
                ),
                method(
                    SoftwarePlatform::Windows,
                    InstallerKind::Curl,
                    "curl 下载",
                    "WeChatSetup.exe",
                    "curl -L https://dldir1.qq.com/weixin/Windows/WeChatSetup.exe -o WeChatSetup.exe",
                    "保留直链下载方案。",
                ),
            ],
        },
        SoftwareEntryInput {
            id: None,
            slug: "docker-desktop".to_string(),
            title: "Docker Desktop".to_string(),
            vendor: "Docker".to_string(),
            summary: "桌面容器工作台，常见于本地开发环境和运维调试。".to_string(),
            homepage_url: "https://www.docker.com/products/docker-desktop/".to_string(),
            icon_url: "https://cdn.simpleicons.org/docker".to_string(),
            trial_platforms: vec![SoftwarePlatform::Macos, SoftwarePlatform::Windows],
            tags: vec!["container".to_string(), "devops".to_string()],
            methods: vec![
                method(
                    SoftwarePlatform::Macos,
                    InstallerKind::Brew,
                    "brew cask",
                    "docker",
                    "brew install --cask docker",
                    "统一 Mac 工作站最常见。",
                ),
                method(
                    SoftwarePlatform::Windows,
                    InstallerKind::Winget,
                    "winget",
                    "Docker.DockerDesktop",
                    "winget install --id Docker.DockerDesktop -e",
                    "Windows 官方分发。",
                ),
            ],
        },
    ]
}

fn method(
    platform: SoftwarePlatform,
    kind: InstallerKind,
    label: &str,
    package_id: &str,
    command: &str,
    note: &str,
) -> SoftwareInstallMethodDto {
    SoftwareInstallMethodDto {
        id: String::new(),
        platform,
        kind,
        label: label.to_string(),
        package_id: package_id.to_string(),
        asset_item_id: None,
        command: command.to_string(),
        note: note.to_string(),
    }
}

fn infer_slug(metadata: &SoftwareMetadataDto) -> SoftwareCatalogResult<String> {
    let mut candidates = Vec::new();
    if !metadata.title.trim().is_empty() {
        candidates.push(metadata.title.trim().to_string());
    }
    if let Ok(url) = Url::parse(&metadata.homepage_url) {
        if let Some(host) = url.host_str() {
            candidates.push(host.to_string());
        }
        let path = url.path().trim_matches('/');
        if !path.is_empty() {
            candidates.push(path.to_string());
        }
    }

    for candidate in candidates {
        let slug = slugify(&candidate);
        if !slug.is_empty() {
            return Ok(slug);
        }
    }

    Err(SoftwareCatalogError::fetch(
        "无法从官网标题或 URL 推断软件 slug",
    ))
}

fn normalized_title(raw_title: &str, slug: &str) -> String {
    let trimmed = raw_title.trim();
    if trimmed.is_empty() {
        return title_case_slug(slug);
    }

    let separators = ['|', '-', '—', '–', '·', ':'];
    let first = trimmed
        .split(separators)
        .next()
        .map(str::trim)
        .unwrap_or(trimmed);
    if first.is_empty() {
        title_case_slug(slug)
    } else {
        first.to_string()
    }
}

fn infer_vendor(metadata: &SoftwareMetadataDto, title: &str) -> String {
    if let Ok(url) = Url::parse(&metadata.homepage_url) {
        if let Some(host) = url.host_str() {
            let host = host.strip_prefix("www.").unwrap_or(host);
            let host_root = host.split('.').next().unwrap_or(host);
            let vendor = title_case_slug(host_root);
            if !vendor.is_empty() && !same_insensitive(&vendor, title) {
                return vendor;
            }
        }
    }

    title.to_string()
}

fn infer_tags(slug: &str, title: &str, vendor: &str) -> Vec<String> {
    let mut tags = Vec::new();
    for keyword in [slug, title, vendor] {
        let normalized = slugify(keyword);
        match normalized.as_str() {
            "cursor" | "vscode" | "visualstudiocode" | "zed" | "windsurf" => {
                push_tag(&mut tags, "ide");
                push_tag(&mut tags, "developer-tools");
            }
            "obsidian" | "notion" | "logseq" => {
                push_tag(&mut tags, "notes");
                push_tag(&mut tags, "knowledge");
            }
            "bun" | "nodejs" | "python" | "uv" => {
                push_tag(&mut tags, "runtime");
            }
            "docker" | "dockerdesktop" | "orbstack" => {
                push_tag(&mut tags, "container");
                push_tag(&mut tags, "devops");
            }
            "wechat" | "qq" | "discord" | "slack" => {
                push_tag(&mut tags, "communication");
            }
            _ => {}
        }
    }

    if tags.is_empty() {
        push_tag(&mut tags, "software");
    }
    tags
}

fn build_methods(
    slug: &str,
    title: &str,
    preferred_platforms: &[SoftwarePlatform],
) -> Vec<SoftwareInstallMethodDto> {
    let platforms = default_platforms(preferred_platforms);
    let mut methods = Vec::new();

    for platform in &platforms {
        match platform {
            SoftwarePlatform::Macos => {
                methods.push(method(
                    SoftwarePlatform::Macos,
                    InstallerKind::Brew,
                    "brew",
                    slug,
                    &format!("brew install --cask {slug}"),
                    "按官网生成的默认 Homebrew 候选，保存前请确认公式或 cask 名称。",
                ));
                methods.push(method(
                    SoftwarePlatform::Macos,
                    InstallerKind::Curl,
                    "curl 下载",
                    &format!("{slug}.dmg"),
                    "",
                    "先抓官网信息生成草稿，再补真实下载链接或关联安装包资产。",
                ));
            }
            SoftwarePlatform::Windows => {
                methods.push(method(
                    SoftwarePlatform::Windows,
                    InstallerKind::Winget,
                    "winget",
                    &format!("Vendor.{title}"),
                    "",
                    "按官网生成的默认 winget 候选，保存前请确认真实 ID。",
                ));
                methods.push(method(
                    SoftwarePlatform::Windows,
                    InstallerKind::Curl,
                    "curl 下载",
                    &format!("{slug}-installer.exe"),
                    "",
                    "给不能使用 winget 的环境预留直链下载入口。",
                ));
            }
            SoftwarePlatform::Linux => {
                methods.push(method(
                    SoftwarePlatform::Linux,
                    InstallerKind::Curl,
                    "curl 下载",
                    &format!("{slug}.tar.gz"),
                    "",
                    "Linux 默认先保留 shell / tarball 安装入口，保存前确认实际分发方式。",
                ));
            }
        }
    }

    methods
}

fn default_platforms(preferred_platforms: &[SoftwarePlatform]) -> Vec<SoftwarePlatform> {
    if preferred_platforms.is_empty() {
        vec![SoftwarePlatform::Macos, SoftwarePlatform::Windows]
    } else {
        preferred_platforms.to_vec()
    }
}

fn best_icon_url(icon_url: &str) -> String {
    if !icon_url.trim().is_empty() {
        return icon_url.trim().to_string();
    }

    String::new()
}

fn push_tag(tags: &mut Vec<String>, value: &str) {
    if !tags.iter().any(|tag| tag == value) {
        tags.push(value.to_string());
    }
}

fn slugify(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn title_case_slug(value: &str) -> String {
    value
        .split(['-', '_', '.'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            let mut word = first.to_uppercase().collect::<String>();
            word.push_str(chars.as_str());
            word
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn same_insensitive(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

fn extract_first(body: &str, re: &Regex) -> Option<String> {
    re.captures(body)?
        .get(1)
        .map(|found| found.as_str().trim().to_string())
}

fn extract_icon_url(body: &str, base_url: &Url) -> Option<String> {
    let href = icon_href_re()
        .captures(body)?
        .get(1)
        .map(|found| found.as_str().trim().to_string())?;
    base_url.join(&href).ok().map(|url| url.to_string())
}

fn default_favicon_url(url: &Url) -> String {
    let mut favicon = url.clone();
    favicon.set_path("/favicon.ico");
    favicon.set_query(None);
    favicon.set_fragment(None);
    favicon.to_string()
}

fn html_unescape(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
}

#[cfg(test)]
mod tests {
    use super::html_unescape;

    #[test]
    fn html_unescape_should_decode_common_entities() {
        assert_eq!(html_unescape("A &amp; B &quot;C&quot;"), "A & B \"C\"");
    }
}
