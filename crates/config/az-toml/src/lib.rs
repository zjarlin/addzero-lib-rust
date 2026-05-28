//! # az-toml
//!
//! Gradle 风格 Version Catalog TOML 文件的解析、构建、序列化与合并工具库。
//!
//! 用于处理 `libs.versions.toml` 格式的 TOML 配置文件，支持：
//!
//! - 从字符串或文件路径解析为 [`VersionCatalog`] 结构体。
//! - 通过 [`VersionCatalog::load_or_init`] 在文件不存在时自动创建默认模板。
//! - 通过 [`VersionCatalog::to_string_pretty`] 格式化输出 TOML 内容。
//! - 通过 [`VersionCatalog::merge_many`] 合并多个 catalog（版本、插件、bundle 首次优先，库按 group+name 去重）。
//! - 通过 [`catalog!`] 宏以声明式 DSL 构建 catalog 值。
//! - 通过 [`insert_after_table`] 在 TOML 文本中按表名定位并插入内容。
//!
//! ## 主要类型
//!
//! | 类型 | 说明 |
//! |------|------|
//! | [`VersionCatalog`] | 顶层 catalog，包含 versions、libraries、plugins、bundles 四个分组 |
//! | [`VersionEntry`] | `[versions]` 条目，包含版本引用名和版本号 |
//! | [`LibraryEntry`] | `[libraries]` 条目，包含 group、name 和 version / version.ref |
//! | [`PluginEntry`] | `[plugins]` 条目，包含插件 id 和 version / version.ref |
//! | [`BundleEntry`] | `[bundles]` 条目，将多个 library key 打包 |
use az_derive_aliases::{
    apply, deserialize_eq, error, impl_from_str_parse, plain_default_eq, plain_eq,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{Array, DocumentMut, Item, Table, Value};

/// `libs.versions.toml` 不存在时写入的默认模板。
///
/// 模板只用于初始化空文件，调用方后续应通过 `VersionCatalog` 结构化读写实际 catalog 内容。
pub const DEFAULT_VERSION_CATALOG_TEMPLATE: &str = r#"[versions]
kotlin = "2.1.0"

[libraries]
hutool = { group = "cn.hutool", name = "hutool-all", version.ref = "kotlin" }

[plugins]
kotlin = { id = "org.jetbrains.kotlin.jvm", version.ref = "kotlin" }

[bundles]
spring = ["spring-boot", "spring-core"]
"#;

/// Version Catalog 文件读写和解析过程中可能返回的错误。
///
/// 该类型保留底层 IO / `toml_edit` 错误链，便于上层区分文件系统失败和 TOML 内容失败。
#[apply(error)]
pub enum TomlCatalogError {
    /// 读取 TOML 文件失败。
    #[error("failed to read TOML file at {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// 写入 TOML 文件失败，通常来自初始化默认模板。
    #[error("failed to write TOML file at {path}: {source}")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// 创建目标父目录失败。
    #[error("failed to create parent directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// TOML 语法或 catalog 结构反序列化失败。
    #[error("failed to parse TOML from {context}: {source}")]
    Parse {
        context: String,
        #[source]
        source: toml_edit::de::Error,
    },
}

/// Gradle Version Catalog 的结构化表示。
///
/// 对应 `libs.versions.toml` 中的 `[versions]`、`[libraries]`、`[plugins]` 和 `[bundles]` 四个顶层分组。
#[apply(plain_default_eq)]
pub struct VersionCatalog {
    /// `[versions]` 分组中的版本别名。
    pub versions: Vec<VersionEntry>,
    /// `[libraries]` 分组中的依赖坐标。
    pub libraries: Vec<LibraryEntry>,
    /// `[plugins]` 分组中的 Gradle 插件坐标。
    pub plugins: Vec<PluginEntry>,
    /// `[bundles]` 分组中的依赖组合。
    pub bundles: Vec<BundleEntry>,
}

/// `[libraries]` 中的单个依赖条目。
///
/// `version` 和 `version_ref` 分别对应 TOML 中的直接版本与 `version.ref` 引用，二者都可能为空。
#[apply(plain_eq)]
pub struct LibraryEntry {
    /// catalog 内部使用的 library key。
    pub key: String,
    /// Maven group 坐标。
    pub group: String,
    /// Maven artifact name。
    pub name: String,
    /// 直接写在条目上的版本号。
    pub version: Option<String>,
    /// 指向 `[versions]` 的版本引用名。
    pub version_ref: Option<String>,
}

/// `[plugins]` 中的单个插件条目。
#[apply(plain_eq)]
pub struct PluginEntry {
    /// catalog 内部使用的 plugin key。
    pub key: String,
    /// Gradle 插件 id。
    pub id: String,
    /// 直接写在条目上的版本号。
    pub version: Option<String>,
    /// 指向 `[versions]` 的版本引用名。
    pub version_ref: Option<String>,
}

/// `[versions]` 中的单个版本别名。
#[apply(plain_eq)]
pub struct VersionEntry {
    /// 版本引用名。
    pub version_ref: String,
    /// 实际版本字符串。
    pub version: String,
}

/// `[bundles]` 中的单个依赖组合。
#[apply(plain_eq)]
pub struct BundleEntry {
    /// bundle key。
    pub key: String,
    /// 该 bundle 引用的 library key 列表。
    pub libraries: Vec<String>,
}

impl VersionCatalog {
    /// 从 TOML 字符串解析 Version Catalog。
    ///
    /// 该方法接受 Gradle 常见的 `version = "..."` 与 `version.ref = "..."` 两种版本选择方式。
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(input: &str) -> Result<Self, TomlCatalogError> {
        Self::parse_catalog(input)
    }

    fn parse_catalog(input: &str) -> Result<Self, TomlCatalogError> {
        let raw: RawCatalog =
            toml_edit::de::from_str(input).map_err(|source| TomlCatalogError::Parse {
                context: "string".to_owned(),
                source,
            })?;

        let versions = raw
            .versions
            .into_iter()
            .map(|(version_ref, version)| VersionEntry {
                version_ref,
                version,
            })
            .collect();
        let libraries = raw
            .libraries
            .into_iter()
            .map(|(key, library)| {
                let (version, version_ref) = split_version_selector(library.version);
                LibraryEntry {
                    key,
                    group: library.group,
                    name: library.name,
                    version,
                    version_ref,
                }
            })
            .collect();
        let plugins = raw
            .plugins
            .into_iter()
            .map(|(key, plugin)| {
                let (version, version_ref) = split_version_selector(plugin.version);
                PluginEntry {
                    key,
                    id: plugin.id,
                    version,
                    version_ref,
                }
            })
            .collect();
        let bundles = raw
            .bundles
            .into_iter()
            .map(|(key, libraries)| BundleEntry { key, libraries })
            .collect();

        Ok(Self {
            versions,
            libraries,
            plugins,
            bundles,
        })
    }

    /// 从文件路径读取并解析 Version Catalog。
    ///
    /// 解析失败时错误上下文会替换为实际文件路径，方便日志定位。
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, TomlCatalogError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(|source| TomlCatalogError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_str(&content).map_err(|error| match error {
            TomlCatalogError::Parse { source, .. } => TomlCatalogError::Parse {
                context: path.display().to_string(),
                source,
            },
            other => other,
        })
    }

    /// 读取 catalog 文件；当文件不存在时先创建默认模板。
    ///
    /// 该方法会创建缺失的父目录，但不会覆盖已经存在的 catalog 文件。
    pub fn load_or_init(path: impl AsRef<Path>) -> Result<Self, TomlCatalogError> {
        let path = path.as_ref();
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|source| TomlCatalogError::CreateDir {
                    path: parent.to_path_buf(),
                    source,
                })?;
            }

            fs::write(path, DEFAULT_VERSION_CATALOG_TEMPLATE).map_err(|source| {
                TomlCatalogError::WriteFile {
                    path: path.to_path_buf(),
                    source,
                }
            })?;
        }

        Self::from_path(path)
    }

    /// 序列化为稳定排序的 TOML 文本。
    ///
    /// 输出会按 key 排序各分组，适合生成可读 diff；它不保留输入文件里的原始注释和空行布局。
    pub fn to_string_pretty(&self) -> String {
        let mut doc = DocumentMut::new();

        if !self.versions.is_empty() {
            let mut table = Table::new();
            let mut entries = self.versions.iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.version_ref.cmp(&right.version_ref));
            for entry in entries {
                table.insert(&entry.version_ref, value_item(entry.version.clone()));
            }
            doc["versions"] = Item::Table(table);
        }

        if !self.libraries.is_empty() {
            let mut table = Table::new();
            let mut entries = self.libraries.iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.key.cmp(&right.key));
            for entry in entries {
                table.insert(&entry.key, build_library_item(entry));
            }
            doc["libraries"] = Item::Table(table);
        }

        if !self.plugins.is_empty() {
            let mut table = Table::new();
            let mut entries = self.plugins.iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.key.cmp(&right.key));
            for entry in entries {
                table.insert(&entry.key, build_plugin_item(entry));
            }
            doc["plugins"] = Item::Table(table);
        }

        if !self.bundles.is_empty() {
            let mut table = Table::new();
            let mut entries = self.bundles.iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.key.cmp(&right.key));
            for entry in entries {
                let mut array = Array::default();
                for library in &entry.libraries {
                    array.push(library.as_str());
                }
                table.insert(&entry.key, Item::Value(Value::Array(array)));
            }
            doc["bundles"] = Item::Table(table);
        }

        doc.to_string()
    }

    /// 合并多个 Version Catalog。
    ///
    /// `versions`、`plugins` 和 `bundles` 采用首次出现优先；`libraries` 按 `(group, name)` 去重，后出现条目覆盖先出现条目。
    pub fn merge_many<I>(catalogs: I) -> Self
    where
        I: IntoIterator<Item = VersionCatalog>,
    {
        let mut versions = BTreeMap::new();
        let mut libraries = BTreeMap::new();
        let mut plugins = BTreeMap::new();
        let mut bundles = BTreeMap::new();

        for catalog in catalogs {
            for entry in catalog.versions {
                versions.entry(entry.version_ref.clone()).or_insert(entry);
            }

            for entry in catalog.libraries {
                libraries.insert((entry.group.clone(), entry.name.clone()), entry);
            }

            for entry in catalog.plugins {
                plugins.entry(entry.id.clone()).or_insert(entry);
            }

            for entry in catalog.bundles {
                bundles.entry(entry.key.clone()).or_insert(entry);
            }
        }

        Self {
            versions: versions.into_values().collect(),
            libraries: libraries.into_values().collect(),
            plugins: plugins.into_values().collect(),
            bundles: bundles.into_values().collect(),
        }
    }

    /// 在 TOML 文本指定表头后插入内容。
    ///
    /// 这是同名自由函数的关联函数包装，便于只通过 `VersionCatalog` 使用本 crate 的场景。
    pub fn insert_after_table(content: &str, tag: &str, append_text: &str) -> String {
        insert_after_table(content, tag, append_text)
    }
}

impl_from_str_parse!(VersionCatalog => TomlCatalogError, VersionCatalog::parse_catalog);

/// 在 TOML 文本指定表头后插入内容。
///
/// `tag` 可以是 `plugins` 或 `[plugins]`。找不到表头或表头行没有换行时返回原文本，不尝试解析或重排 TOML。
pub fn insert_after_table(content: &str, tag: &str, append_text: &str) -> String {
    let normalized_tag = if tag.starts_with('[') {
        tag.to_owned()
    } else {
        format!("[{tag}]")
    };

    let Some(tag_index) = content.find(&normalized_tag) else {
        return content.to_owned();
    };

    let Some(relative_newline) = content[tag_index..].find('\n') else {
        return content.to_owned();
    };

    let insert_at = tag_index + relative_newline + 1;
    let mut result = String::with_capacity(content.len() + append_text.len() + 1);
    result.push_str(&content[..insert_at]);
    result.push_str(append_text);
    result.push('\n');
    result.push_str(&content[insert_at..]);
    result
}

fn split_version_selector(
    selector: Option<RawVersionSelector>,
) -> (Option<String>, Option<String>) {
    match selector {
        Some(RawVersionSelector::Direct(version)) => (Some(version), None),
        Some(RawVersionSelector::Reference { r#ref }) => (None, Some(r#ref)),
        None => (None, None),
    }
}

fn value_item(value: impl Into<Value>) -> Item {
    Item::Value(value.into())
}

fn build_library_item(entry: &LibraryEntry) -> Item {
    let mut parts = vec![
        format!("group = {}", encode_string(&entry.group)),
        format!("name = {}", encode_string(&entry.name)),
    ];
    if let Some(version) = &entry.version {
        parts.push(format!("version = {}", encode_string(version)));
    }
    if let Some(version_ref) = &entry.version_ref {
        parts.push(format!("version.ref = {}", encode_string(version_ref)));
    }
    parse_inline_table(&parts.join(", "))
}

fn build_plugin_item(entry: &PluginEntry) -> Item {
    let mut parts = vec![format!("id = {}", encode_string(&entry.id))];
    if let Some(version) = &entry.version {
        parts.push(format!("version = {}", encode_string(version)));
    }
    if let Some(version_ref) = &entry.version_ref {
        parts.push(format!("version.ref = {}", encode_string(version_ref)));
    }
    parse_inline_table(&parts.join(", "))
}

fn encode_string(value: &str) -> String {
    Value::from(value.to_owned()).to_string()
}

fn parse_inline_table(content: &str) -> Item {
    let source = format!("value = {{ {content} }}");
    let document = source
        .parse::<DocumentMut>()
        .expect("internal inline table generation must stay valid TOML");
    document["value"].clone()
}

#[apply(deserialize_eq)]
struct RawCatalog {
    #[serde(default)]
    versions: BTreeMap<String, String>,
    #[serde(default)]
    libraries: BTreeMap<String, RawLibrary>,
    #[serde(default)]
    plugins: BTreeMap<String, RawPlugin>,
    #[serde(default)]
    bundles: BTreeMap<String, Vec<String>>,
}

#[apply(deserialize_eq)]
struct RawLibrary {
    group: String,
    name: String,
    #[serde(default)]
    version: Option<RawVersionSelector>,
}

#[apply(deserialize_eq)]
struct RawPlugin {
    id: String,
    #[serde(default)]
    version: Option<RawVersionSelector>,
}

#[apply(deserialize_eq)]
#[serde(untagged)]
enum RawVersionSelector {
    Direct(String),
    Reference { r#ref: String },
}

/// 内联构造 `VersionCatalog` 的声明式宏。
///
/// 宏语法贴近 `libs.versions.toml` 的四个分组，适合测试、模板和小型生成器；复杂场景仍建议从 TOML 文本解析。
#[macro_export]
macro_rules! catalog {
    () => {
        $crate::VersionCatalog::default()
    };
    ($($section:ident { $($content:tt)* })* $(,)?) => {{
        let mut catalog = $crate::VersionCatalog::default();
        $( $crate::catalog!(@section catalog, $section, { $($content)* }); )*
        catalog
    }};
    (@section $catalog:ident, versions, { $($key:ident = $value:expr),* $(,)? }) => {
        $catalog.versions = vec![
            $(
                $crate::VersionEntry {
                    version_ref: ::std::string::String::from(::core::stringify!($key)),
                    version: ::std::convert::Into::into($value),
                }
            ),*
        ];
    };
    (@section $catalog:ident, libraries, { $($key:ident = { group: $group:expr, name: $name:expr $(, version: $version:expr)? $(, version_ref: $version_ref:expr)? }),* $(,)? }) => {
        $catalog.libraries = vec![
            $(
                $crate::LibraryEntry {
                    key: ::std::string::String::from(::core::stringify!($key)),
                    group: ::std::convert::Into::into($group),
                    name: ::std::convert::Into::into($name),
                    version: $crate::catalog!(@optional_string $($version)?),
                    version_ref: $crate::catalog!(@optional_string $($version_ref)?),
                }
            ),*
        ];
    };
    (@section $catalog:ident, plugins, { $($key:ident = { id: $id:expr $(, version: $version:expr)? $(, version_ref: $version_ref:expr)? }),* $(,)? }) => {
        $catalog.plugins = vec![
            $(
                $crate::PluginEntry {
                    key: ::std::string::String::from(::core::stringify!($key)),
                    id: ::std::convert::Into::into($id),
                    version: $crate::catalog!(@optional_string $($version)?),
                    version_ref: $crate::catalog!(@optional_string $($version_ref)?),
                }
            ),*
        ];
    };
    (@section $catalog:ident, bundles, { $($key:ident = [$($library:expr),* $(,)?]),* $(,)? }) => {
        $catalog.bundles = vec![
            $(
                $crate::BundleEntry {
                    key: ::std::string::String::from(::core::stringify!($key)),
                    libraries: vec![$(::std::convert::Into::into($library)),*],
                }
            ),*
        ];
    };
    (@optional_string) => {
        None
    };
    (@optional_string $value:expr) => {
        Some(::std::convert::Into::into($value))
    };
}
