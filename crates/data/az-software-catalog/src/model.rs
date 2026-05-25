use std::{collections::BTreeSet, fmt};

use az_derive_aliases::{apply, error_eq, serde_code_ord_display_enum, serde_eq, serde_eq_default};
use uuid::Uuid;

#[apply(serde_code_ord_display_enum)]
pub enum SoftwarePlatform {
    #[display("macOS")]
    Macos,
    #[display("Windows")]
    Windows,
    #[display("Linux")]
    Linux,
}

#[apply(serde_code_ord_display_enum)]
pub enum InstallerKind {
    #[display("Homebrew")]
    Brew,
    #[display("Bun")]
    Bun,
    #[display("winget")]
    Winget,
    #[display("Scoop")]
    Scoop,
    #[display("Chocolatey")]
    Choco,
    #[display("curl 下载")]
    Curl,
    #[serde(rename = "package")]
    #[strum(serialize = "package")]
    #[display("安装包")]
    DirectPackage,
    #[display("自定义")]
    Custom,
}

#[apply(serde_eq)]
pub struct SoftwareInstallMethodDto {
    pub id: String,
    pub platform: SoftwarePlatform,
    pub kind: InstallerKind,
    pub label: String,
    pub package_id: String,
    pub asset_item_id: Option<String>,
    pub command: String,
    pub note: String,
}

#[apply(serde_eq)]
pub struct SoftwareEntryDto {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub vendor: String,
    pub summary: String,
    pub homepage_url: String,
    pub icon_url: String,
    pub trial_platforms: Vec<SoftwarePlatform>,
    pub tags: Vec<String>,
    pub methods: Vec<SoftwareInstallMethodDto>,
}

#[apply(serde_eq)]
pub struct SoftwareCatalogDto {
    pub host_platform: SoftwarePlatform,
    pub items: Vec<SoftwareEntryDto>,
}

#[apply(serde_eq_default)]
pub struct SoftwareEntryInput {
    pub id: Option<String>,
    pub slug: String,
    pub title: String,
    pub vendor: String,
    pub summary: String,
    pub homepage_url: String,
    pub icon_url: String,
    pub trial_platforms: Vec<SoftwarePlatform>,
    pub tags: Vec<String>,
    pub methods: Vec<SoftwareInstallMethodDto>,
}

#[apply(serde_eq_default)]
pub struct SoftwareMetadataFetchInput {
    pub homepage_url: String,
}

#[apply(serde_eq_default)]
pub struct SoftwareMetadataDto {
    pub title: String,
    pub summary: String,
    pub homepage_url: String,
    pub icon_url: String,
}

#[apply(serde_eq_default)]
pub struct SoftwareDraftInput {
    pub homepage_url: String,
    pub preferred_platforms: Vec<SoftwarePlatform>,
}

#[apply(error_eq)]
pub enum SoftwareCatalogError {
    #[error("connect software catalog persistence: {0}")]
    Persistence(String),
    #[error("query software catalog rows: {0}")]
    Query(String),
    #[error("fetch software metadata: {0}")]
    Fetch(String),
    #[error("{0}")]
    Message(String),
}

impl SoftwareCatalogError {
    pub(crate) fn persistence(err: impl fmt::Display) -> Self {
        Self::Persistence(err.to_string())
    }

    pub(crate) fn query(err: impl fmt::Display) -> Self {
        Self::Query(err.to_string())
    }

    pub(crate) fn fetch(err: impl fmt::Display) -> Self {
        Self::Fetch(err.to_string())
    }
}

pub type SoftwareCatalogResult<T> = Result<T, SoftwareCatalogError>;

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

pub(crate) fn parse_uuid(value: &str) -> SoftwareCatalogResult<Uuid> {
    Uuid::parse_str(value).map_err(|err| SoftwareCatalogError::Message(format!("非法 UUID：{err}")))
}

pub(crate) fn validate_input(input: &SoftwareEntryInput) -> SoftwareCatalogResult<()> {
    if input.slug.trim().is_empty() || input.title.trim().is_empty() {
        return Err(SoftwareCatalogError::Message(
            "软件 slug 和标题不能为空。".to_string(),
        ));
    }
    Ok(())
}

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

pub(crate) fn clean_tags(tags: &[String]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    tags.iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .filter(|tag| seen.insert(tag.clone()))
        .collect()
}

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

        assert_eq!(InstallerKind::DirectPackage.code(), "package");
        assert_eq!(InstallerKind::DirectPackage.to_string(), "安装包");
        assert_eq!(
            InstallerKind::from_code("package"),
            Some(InstallerKind::DirectPackage)
        );
        assert_eq!(
            serde_json::to_string(&InstallerKind::DirectPackage)
                .expect("installer kind should serialize"),
            "\"package\""
        );
    }
}
