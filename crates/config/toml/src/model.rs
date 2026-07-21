//! Gradle Version Catalog 数据模型。


/// `libs.versions.toml` 不存在时写入的默认模板。
///
/// 模板只用于初始化空文件，调用方后续应通过 [`VersionCatalog`] 结构化读写实际 catalog 内容。
pub const DEFAULT_VERSION_CATALOG_TEMPLATE: &str = r#"[versions]
kotlin = "2.1.0"

[libraries]
hutool = { group = "cn.hutool", name = "hutool-all", version.ref = "kotlin" }

[plugins]
kotlin = { id = "org.jetbrains.kotlin.jvm", version.ref = "kotlin" }

[bundles]
spring = ["spring-boot", "spring-core"]
"#;

/// Gradle Version Catalog 的结构化表示。
///
/// 对应 `libs.versions.toml` 中的 `[versions]`、`[libraries]`、`[plugins]` 和 `[bundles]` 四个顶层分组。
#[derive(Clone, Debug, Default, Eq, PartialEq)]
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
#[derive(Clone, Debug, Eq, PartialEq)]
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
#[derive(Clone, Debug, Eq, PartialEq)]
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionEntry {
    /// 版本引用名。
    pub version_ref: String,
    /// 实际版本字符串。
    pub version: String,
}

/// `[bundles]` 中的单个依赖组合。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundleEntry {
    /// bundle key。
    pub key: String,
    /// 该 bundle 引用的 library key 列表。
    pub libraries: Vec<String>,
}

/// 内联构造 [`VersionCatalog`] 的声明式宏。
///
/// 宏语法贴近 `libs.versions.toml` 的四个分组，适合测试、模板和小型生成器；复杂场景仍建议从 TOML 文本解析。
#[macro_export]
macro_rules! catalog {
    () => {
        $crate::model::VersionCatalog::default()
    };
    ($($section:ident { $($content:tt)* })* $(,)?) => {{
        let mut catalog = $crate::model::VersionCatalog::default();
        $( $crate::catalog!(@section catalog, $section, { $($content)* }); )*
        catalog
    }};
    (@section $catalog:ident, versions, { $($key:ident = $value:expr),* $(,)? }) => {
        $catalog.versions = vec![
            $(
                $crate::model::VersionEntry {
                    version_ref: ::std::string::String::from(::core::stringify!($key)),
                    version: ::std::convert::Into::into($value),
                }
            ),*
        ];
    };
    (@section $catalog:ident, libraries, { $($key:ident = { group: $group:expr, name: $name:expr $(, version: $version:expr)? $(, version_ref: $version_ref:expr)? }),* $(,)? }) => {
        $catalog.libraries = vec![
            $(
                $crate::model::LibraryEntry {
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
                $crate::model::PluginEntry {
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
                $crate::model::BundleEntry {
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
