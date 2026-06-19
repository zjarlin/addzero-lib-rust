//! TOML 反序列化中间结构。

use std::collections::BTreeMap;


use crate::model::{BundleEntry, LibraryEntry, PluginEntry, VersionCatalog, VersionEntry};

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub(crate) struct RawCatalog {
    #[serde(default)]
    versions: BTreeMap<String, String>,
    #[serde(default)]
    libraries: BTreeMap<String, RawLibrary>,
    #[serde(default)]
    plugins: BTreeMap<String, RawPlugin>,
    #[serde(default)]
    bundles: BTreeMap<String, Vec<String>>,
}

impl RawCatalog {
    pub(crate) fn into_catalog(self) -> VersionCatalog {
        let versions = self
            .versions
            .into_iter()
            .map(|(version_ref, version)| VersionEntry {
                version_ref,
                version,
            })
            .collect();
        let libraries = self
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
        let plugins = self
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
        let bundles = self
            .bundles
            .into_iter()
            .map(|(key, libraries)| BundleEntry { key, libraries })
            .collect();

        VersionCatalog {
            versions,
            libraries,
            plugins,
            bundles,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
struct RawLibrary {
    group: String,
    name: String,
    #[serde(default)]
    version: Option<RawVersionSelector>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
struct RawPlugin {
    id: String,
    #[serde(default)]
    version: Option<RawVersionSelector>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(untagged)]
enum RawVersionSelector {
    Direct(String),
    Reference { r#ref: String },
}

fn split_version_selector(selector: Option<RawVersionSelector>) -> (Option<String>, Option<String>) {
    match selector {
        Some(RawVersionSelector::Direct(version)) => (Some(version), None),
        Some(RawVersionSelector::Reference { r#ref }) => (None, Some(r#ref)),
        None => (None, None),
    }
}
