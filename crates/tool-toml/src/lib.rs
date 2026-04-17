use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;
use toml_edit::{Array, DocumentMut, Item, Table, Value};

pub const DEFAULT_VERSION_CATALOG_TEMPLATE: &str = r#"[versions]
kotlin = "2.1.0"

[libraries]
hutool = { group = "cn.hutool", name = "hutool-all", version.ref = "kotlin" }

[plugins]
kotlin = { id = "org.jetbrains.kotlin.jvm", version.ref = "kotlin" }

[bundles]
spring = ["spring-boot", "spring-core"]
"#;

#[derive(Debug, Error)]
pub enum TomlCatalogError {
    #[error("failed to read TOML file at {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write TOML file at {path}: {source}")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to create parent directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse TOML from {context}: {source}")]
    Parse {
        context: String,
        #[source]
        source: toml_edit::de::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VersionCatalog {
    pub versions: Vec<VersionEntry>,
    pub libraries: Vec<LibraryEntry>,
    pub plugins: Vec<PluginEntry>,
    pub bundles: Vec<BundleEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryEntry {
    pub key: String,
    pub group: String,
    pub name: String,
    pub version: Option<String>,
    pub version_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginEntry {
    pub key: String,
    pub id: String,
    pub version: Option<String>,
    pub version_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionEntry {
    pub version_ref: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleEntry {
    pub key: String,
    pub libraries: Vec<String>,
}

impl VersionCatalog {
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

    pub fn insert_after_table(content: &str, tag: &str, append_text: &str) -> String {
        insert_after_table(content, tag, append_text)
    }
}

impl FromStr for VersionCatalog {
    type Err = TomlCatalogError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse_catalog(value)
    }
}

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

#[derive(Debug, Deserialize, Default)]
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

#[derive(Debug, Deserialize)]
struct RawLibrary {
    group: String,
    name: String,
    #[serde(default)]
    version: Option<RawVersionSelector>,
}

#[derive(Debug, Deserialize)]
struct RawPlugin {
    id: String,
    #[serde(default)]
    version: Option<RawVersionSelector>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawVersionSelector {
    Direct(String),
    Reference { r#ref: String },
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn parses_and_serializes_version_catalog() {
        let source = r#"[versions]
kotlin = "2.1.0"

[libraries]
hutool = { group = "cn.hutool", name = "hutool-all", version.ref = "kotlin" }

[plugins]
kotlin = { id = "org.jetbrains.kotlin.jvm", version.ref = "kotlin" }

[bundles]
spring = ["spring-boot", "spring-core"]
"#;

        let catalog = VersionCatalog::from_str(source).expect("catalog should parse");
        let expected = catalog! {
            versions {
                kotlin = "2.1.0",
            }
            libraries {
                hutool = { group: "cn.hutool", name: "hutool-all", version_ref: "kotlin" },
            }
            plugins {
                kotlin = { id: "org.jetbrains.kotlin.jvm", version_ref: "kotlin" },
            }
            bundles {
                spring = ["spring-boot", "spring-core"],
            }
        };

        assert_eq!(catalog, expected);

        let rendered = catalog.to_string_pretty();
        assert!(rendered.contains("version.ref = \"kotlin\""));
        let reparsed = VersionCatalog::from_str(&rendered).expect("rendered catalog should parse");
        assert_eq!(reparsed, expected);
    }

    #[test]
    fn load_or_init_creates_default_catalog_only_when_requested() {
        let temp = TempDir::new().expect("temp dir should be created");
        let path = temp.path().join("gradle/libs.versions.toml");

        let catalog = VersionCatalog::load_or_init(&path).expect("catalog should load");

        assert!(path.exists());
        assert_eq!(
            catalog,
            VersionCatalog::from_str(DEFAULT_VERSION_CATALOG_TEMPLATE)
                .expect("default template should parse")
        );
    }

    #[test]
    fn merge_many_uses_jvm_compatibility_rules() {
        let left = catalog! {
            versions { kotlin = "2.1.0" }
            libraries {
                hutool = { group: "cn.hutool", name: "hutool-all", version_ref: "kotlin" },
            }
            plugins {
                kotlin = { id: "org.jetbrains.kotlin.jvm", version_ref: "kotlin" },
            }
            bundles {
                spring = ["spring-boot"],
            }
        };
        let right = catalog! {
            versions { kotlin = "2.2.0", serde = "1.0.0" }
            libraries {
                hutool = { group: "cn.hutool", name: "hutool-all", version: "6.0.0" },
                serde = { group: "serde", name: "serde", version_ref: "serde" },
            }
            plugins {
                kotlin_android = { id: "org.jetbrains.kotlin.jvm", version: "2.2.0" },
                android = { id: "com.android.application", version: "8.8.0" },
            }
            bundles {
                spring = ["spring-boot", "spring-core"],
                common = ["serde"],
            }
        };

        let merged = VersionCatalog::merge_many(vec![left, right]);

        assert_eq!(
            merged.versions,
            vec![
                VersionEntry {
                    version_ref: "kotlin".to_owned(),
                    version: "2.1.0".to_owned(),
                },
                VersionEntry {
                    version_ref: "serde".to_owned(),
                    version: "1.0.0".to_owned(),
                },
            ]
        );
        assert_eq!(merged.libraries.len(), 2);
        assert_eq!(merged.libraries[0].group, "cn.hutool");
        assert_eq!(merged.libraries[0].version.as_deref(), Some("6.0.0"));
        assert_eq!(merged.plugins.len(), 2);
        assert_eq!(merged.plugins[0].id, "com.android.application");
        assert_eq!(merged.plugins[1].id, "org.jetbrains.kotlin.jvm");
        assert_eq!(merged.bundles.len(), 2);
        assert_eq!(merged.bundles[1].key, "spring");
        assert_eq!(merged.bundles[1].libraries, vec!["spring-boot".to_owned()]);
    }

    #[test]
    fn insert_after_table_supports_bare_and_bracket_tags() {
        let source = "[plugins]\nkotlin = { id = \"org.jetbrains.kotlin.jvm\" }\n";

        let inserted_bare = insert_after_table(
            source,
            "plugins",
            "android = { id = \"com.android.application\" }",
        );
        let inserted_bracket = insert_after_table(
            source,
            "[plugins]",
            "android = { id = \"com.android.application\" }",
        );

        assert_eq!(inserted_bare, inserted_bracket);
        assert!(inserted_bare.contains("android = { id = \"com.android.application\" }"));
    }

    #[test]
    fn catalog_macro_matches_parser_output() {
        let macro_catalog = catalog! {
            versions {
                kotlin = "2.1.0",
            }
            libraries {
                hutool = { group: "cn.hutool", name: "hutool-all", version_ref: "kotlin" },
            }
            plugins {
                kotlin = { id: "org.jetbrains.kotlin.jvm", version_ref: "kotlin" },
            }
            bundles {
                spring = ["spring-boot", "spring-core"],
            }
        };

        let parsed = VersionCatalog::from_str(&macro_catalog.to_string_pretty())
            .expect("macro catalog should round-trip");

        assert_eq!(parsed, macro_catalog);
    }
}
