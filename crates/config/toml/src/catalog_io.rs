//! Version catalog parsing, loading, and merging.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::insert_after_table::insert_after_table;
use crate::model::{DEFAULT_VERSION_CATALOG_TEMPLATE, VersionCatalog};
use crate::raw_catalog::RawCatalog;
use crate::toml_render::render_pretty_catalog;

impl VersionCatalog {
    /// 从 TOML 字符串解析 Version Catalog。
    ///
    /// 该方法接受 Gradle 常见的 `version = "..."` 与 `version.ref = "..."` 两种版本选择方式。
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(input: &str) -> Result<Self> {
        Self::parse_catalog(input)
    }

    pub(crate) fn parse_catalog(input: &str) -> Result<Self> {
        let raw: RawCatalog =
            toml_edit::de::from_str(input).context("failed to parse TOML from string")?;

        Ok(raw.into_catalog())
    }

    /// 从文件路径读取并解析 Version Catalog。
    ///
    /// 解析失败时错误上下文会替换为实际文件路径，方便日志定位。
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read TOML file at {}", path.display()))?;
        toml_edit::de::from_str::<RawCatalog>(&content)
            .with_context(|| format!("failed to parse TOML from {}", path.display()))
            .map(RawCatalog::into_catalog)
    }

    /// 读取 catalog 文件；当文件不存在时先创建默认模板。
    ///
    /// 该方法会创建缺失的父目录，但不会覆盖已经存在的 catalog 文件。
    pub fn load_or_init(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("failed to create parent directory {}", parent.display())
                })?;
            }

            fs::write(path, DEFAULT_VERSION_CATALOG_TEMPLATE)
                .with_context(|| format!("failed to write TOML file at {}", path.display()))?;
        }

        Self::from_path(path)
    }

    /// 序列化为稳定排序的 TOML 文本。
    ///
    /// 输出会按 key 排序各分组，适合生成可读 diff；它不保留输入文件里的原始注释和空行布局。
    pub fn to_string_pretty(&self) -> String {
        render_pretty_catalog(self)
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

impl std::str::FromStr for VersionCatalog {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        Self::parse_catalog(value)
    }
}
