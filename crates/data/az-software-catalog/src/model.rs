#[cfg(any(not(target_arch = "wasm32"), test))]
use std::collections::BTreeSet;

#[cfg(not(target_arch = "wasm32"))]
use anyhow::{Context, bail};
#[cfg(any(not(target_arch = "wasm32"), test))]
use uuid::Uuid;

/// 软件可试用或可安装的平台。
///
/// `code()` 和 serde wire value 使用稳定小写值，`Display` 用于界面展示。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, PartialOrd, Ord, Default, derive_more::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum SoftwarePlatform {
    /// macOS 平台。
    #[default]
    #[display("macOS")]
    Macos,
    /// Windows 平台。
    #[display("Windows")]
    Windows,
    /// Linux 平台。
    #[display("Linux")]
    Linux,
}

impl SoftwarePlatform {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }

    #[must_use]
    pub fn from_code_or_default(value: &str) -> Self {
        Self::from_code(value).unwrap_or_default()
    }
}

/// 软件安装方式类型。
///
/// 这里的 code 是前后端和持久化共享的安装器类别，不等同于具体命令文本。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, PartialOrd, Ord, Default, derive_more::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum InstallerKind {
    /// Homebrew 安装。
    #[display("Homebrew")]
    Brew,
    /// Bun 工具链安装。
    #[display("Bun")]
    Bun,
    /// Windows winget 安装。
    #[display("winget")]
    Winget,
    /// Windows Scoop 安装。
    #[display("Scoop")]
    Scoop,
    /// Windows Chocolatey 安装。
    #[display("Chocolatey")]
    Choco,
    /// 通过 curl 或直链脚本下载。
    #[display("curl 下载")]
    Curl,
    /// 直接下载安装包文件；wire value 保持为历史兼容的 `package`。
    #[serde(rename = "package")]
    #[strum(serialize = "package")]
    #[display("安装包")]
    DirectPackage,
    /// 无法归入固定安装器的自定义命令。
    #[default]
    #[display("自定义")]
    Custom,
}

impl InstallerKind {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }

    #[must_use]
    pub fn from_code_or_default(value: &str) -> Self {
        Self::from_code(value).unwrap_or_default()
    }
}

/// 单个软件安装方法。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SoftwareInstallMethodDto {
    /// 安装方法 ID，缺失时保存流程会生成 UUID。
    pub id: String,
    /// 该方法适用的平台。
    pub platform: SoftwarePlatform,
    /// 安装器类别。
    pub kind: InstallerKind,
    /// 展示标签，例如 `brew` 或 `官方安装包`。
    pub label: String,
    /// 包管理器中的包名、安装包标识或下载标识。
    pub package_id: String,
    /// 关联资产库条目的可选 ID。
    pub asset_item_id: Option<String>,
    /// 实际安装命令或下载命令。
    pub command: String,
    /// 额外说明。
    pub note: String,
}

/// 软件目录中的单个软件条目。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SoftwareEntryDto {
    /// 软件条目 ID。
    pub id: String,
    /// 人类可读且 URL 友好的稳定 slug。
    pub slug: String,
    /// 软件名称。
    pub title: String,
    /// 厂商或维护者。
    pub vendor: String,
    /// 简短说明。
    pub summary: String,
    /// 官方主页 URL。
    pub homepage_url: String,
    /// 图标 URL。
    pub icon_url: String,
    /// 已验证或计划试用的平台。
    pub trial_platforms: Vec<SoftwarePlatform>,
    /// 搜索和分组标签。
    pub tags: Vec<String>,
    /// 可用安装方法列表。
    pub methods: Vec<SoftwareInstallMethodDto>,
}

/// 软件目录查询响应。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SoftwareCatalogDto {
    /// 当前服务进程识别出的宿主平台。
    pub host_platform: SoftwarePlatform,
    /// 软件条目列表。
    pub items: Vec<SoftwareEntryDto>,
}

/// 创建或更新软件条目的输入。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SoftwareEntryInput {
    /// 已存在条目的 ID；为空时保存流程会创建新条目。
    pub id: Option<String>,
    /// URL 友好的稳定 slug，保存前会去除首尾空白。
    pub slug: String,
    /// 软件名称，不能为空。
    pub title: String,
    /// 厂商或维护者。
    pub vendor: String,
    /// 简短说明。
    pub summary: String,
    /// 官方主页 URL。
    pub homepage_url: String,
    /// 图标 URL。
    pub icon_url: String,
    /// 试用平台列表，保存前会去重。
    pub trial_platforms: Vec<SoftwarePlatform>,
    /// 标签列表，保存前会 trim 并去重。
    pub tags: Vec<String>,
    /// 安装方法列表，保存前会丢弃完全空白的方法。
    pub methods: Vec<SoftwareInstallMethodDto>,
}

/// 从软件主页抓取元数据的输入。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SoftwareMetadataFetchInput {
    /// 软件官方主页 URL。
    pub homepage_url: String,
}

/// 从软件主页推断出的基础元数据。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SoftwareMetadataDto {
    /// 推断出的软件名称。
    pub title: String,
    /// 推断出的简短说明。
    pub summary: String,
    /// 规范化后的主页 URL。
    pub homepage_url: String,
    /// 推断出的图标 URL。
    pub icon_url: String,
}

/// 构建软件草稿时的输入。
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SoftwareDraftInput {
    /// 软件官方主页 URL。
    pub homepage_url: String,
    /// 草稿优先生成安装方法的平台。
    pub preferred_platforms: Vec<SoftwarePlatform>,
}

/// 返回当前编译目标对应的平台。
pub fn current_platform() -> SoftwarePlatform {
    #[cfg(target_os = "windows")]
    {
        return SoftwarePlatform::Windows;
    }

    #[cfg(target_os = "linux")]
    {
        return SoftwarePlatform::Linux;
    }

    SoftwarePlatform::Macos
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn parse_uuid(value: &str) -> anyhow::Result<Uuid> {
    Uuid::parse_str(value).with_context(|| format!("解析软件条目 UUID `{value}`"))
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn validate_input(input: &SoftwareEntryInput) -> anyhow::Result<()> {
    if input.slug.trim().is_empty() || input.title.trim().is_empty() {
        bail!("软件 slug 和标题不能为空。");
    }
    Ok(())
}

#[cfg(any(not(target_arch = "wasm32"), test))]
pub(crate) fn normalize_input(input: SoftwareEntryInput) -> SoftwareEntryDto {
    SoftwareEntryDto {
        id: input.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
        slug: input.slug.trim().to_string(),
        title: input.title.trim().to_string(),
        vendor: input.vendor.trim().to_string(),
        summary: input.summary.trim().to_string(),
        homepage_url: input.homepage_url.trim().to_string(),
        icon_url: input.icon_url.trim().to_string(),
        trial_platforms: clean_platforms(&input.trial_platforms),
        tags: clean_tags(&input.tags),
        methods: input
            .methods
            .into_iter()
            .filter(|method| {
                !method.label.trim().is_empty()
                    || !method.package_id.trim().is_empty()
                    || method
                        .asset_item_id
                        .as_deref()
                        .is_some_and(|value| !value.trim().is_empty())
                    || !method.command.trim().is_empty()
            })
            .map(|mut method| {
                if method.id.trim().is_empty() {
                    method.id = Uuid::new_v4().to_string();
                }
                method.label = method.label.trim().to_string();
                method.package_id = method.package_id.trim().to_string();
                method.asset_item_id = method.asset_item_id.and_then(|value| {
                    let trimmed = value.trim().to_string();
                    (!trimmed.is_empty()).then_some(trimmed)
                });
                method.command = method.command.trim().to_string();
                method.note = method.note.trim().to_string();
                method
            })
            .collect(),
    }
}

#[cfg(any(not(target_arch = "wasm32"), test))]
pub(crate) fn clean_tags(tags: &[String]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    tags.iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .filter(|tag| seen.insert(tag.clone()))
        .collect()
}

#[cfg(any(not(target_arch = "wasm32"), test))]
pub(crate) fn clean_platforms(platforms: &[SoftwarePlatform]) -> Vec<SoftwarePlatform> {
    let mut seen = BTreeSet::new();
    platforms
        .iter()
        .copied()
        .filter(|platform| seen.insert(platform.code()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        InstallerKind, SoftwareEntryInput, SoftwareInstallMethodDto, SoftwarePlatform,
        clean_platforms, clean_tags, normalize_input,
    };

    #[test]
    fn normalize_input_should_trim_and_generate_missing_ids() {
        let normalized = normalize_input(SoftwareEntryInput {
            id: None,
            slug: " cursor ".to_string(),
            title: " Cursor ".to_string(),
            vendor: " Anysphere ".to_string(),
            summary: " AI IDE ".to_string(),
            homepage_url: " https://cursor.com ".to_string(),
            icon_url: " https://cdn.simpleicons.org/cursor ".to_string(),
            trial_platforms: vec![
                SoftwarePlatform::Macos,
                SoftwarePlatform::Macos,
                SoftwarePlatform::Windows,
            ],
            tags: vec![" ide ".to_string(), "agent".to_string(), "ide".to_string()],
            methods: vec![SoftwareInstallMethodDto {
                id: String::new(),
                platform: SoftwarePlatform::Macos,
                kind: InstallerKind::Brew,
                label: " brew ".to_string(),
                package_id: " cursor ".to_string(),
                asset_item_id: Some(" asset-1 ".to_string()),
                command: " brew install cursor ".to_string(),
                note: " note ".to_string(),
            }],
        });

        assert_eq!(normalized.slug, "cursor");
        assert_eq!(normalized.title, "Cursor");
        assert_eq!(normalized.vendor, "Anysphere");
        assert_eq!(normalized.summary, "AI IDE");
        assert_eq!(normalized.trial_platforms.len(), 2);
        assert_eq!(
            normalized.tags,
            vec!["ide".to_string(), "agent".to_string()]
        );
        assert!(!normalized.id.is_empty());
        assert!(!normalized.methods[0].id.is_empty());
        assert_eq!(normalized.methods[0].label, "brew");
        assert_eq!(
            normalized.methods[0].asset_item_id,
            Some("asset-1".to_string())
        );
    }

    #[test]
    fn clean_helpers_should_deduplicate_values() {
        let tags = clean_tags(&[
            "notes".to_string(),
            " notes ".to_string(),
            "agent".to_string(),
        ]);
        let platforms = clean_platforms(&[
            SoftwarePlatform::Linux,
            SoftwarePlatform::Linux,
            SoftwarePlatform::Macos,
        ]);

        assert_eq!(tags, vec!["notes".to_string(), "agent".to_string()]);
        assert_eq!(
            platforms,
            vec![SoftwarePlatform::Linux, SoftwarePlatform::Macos]
        );
    }

    #[test]
    fn code_enums_keep_storage_values() {
        assert_eq!(SoftwarePlatform::Macos.code(), "macos");
        assert_eq!(SoftwarePlatform::Macos.to_string(), "macOS");
        assert_eq!(
            SoftwarePlatform::from_code("linux"),
            Some(SoftwarePlatform::Linux)
        );
        assert_eq!(
            SoftwarePlatform::from_code_or_default("unknown"),
            SoftwarePlatform::Macos
        );

        assert_eq!(InstallerKind::DirectPackage.code(), "package");
        assert_eq!(InstallerKind::DirectPackage.to_string(), "安装包");
        assert_eq!(
            InstallerKind::from_code("package"),
            Some(InstallerKind::DirectPackage)
        );
        assert_eq!(
            InstallerKind::from_code_or_default("unknown"),
            InstallerKind::Custom
        );
        assert_eq!(
            serde_json::to_string(&InstallerKind::DirectPackage)
                .expect("installer kind should serialize"),
            "\"package\""
        );
    }
}
