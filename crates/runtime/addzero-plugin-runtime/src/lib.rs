use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use addzero_plugin_contract::{
    MarketplaceEntry, MarketplaceSnapshot, PluginDescriptor, PluginInstance, PluginInstanceConfig,
    PluginPackageManifest, PluginStatus,
};
use chrono::Utc;
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("plugin package `{0}` not found in catalog")]
    PackageNotFound(String),
    #[error("plugin `{0}` is already installed")]
    AlreadyInstalled(String),
    #[error("plugin `{0}` is not installed")]
    NotInstalled(String),
    #[error("plugin `{plugin_id}` page `{page_id}` was not found")]
    PageNotFound { plugin_id: String, page_id: String },
    #[error("plugin instance `{0}` was not found")]
    InstanceNotFound(String),
    #[error("plugin package is invalid: {0}")]
    InvalidPackage(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error(transparent)]
    Toml(#[from] toml_edit::de::Error),
}

#[derive(Clone, Debug)]
pub struct CatalogPlugin {
    pub manifest: PluginPackageManifest,
    pub package_path: PathBuf,
}

#[derive(Clone, Debug)]
pub struct InstalledPlugin {
    pub manifest: PluginPackageManifest,
    pub install_dir: PathBuf,
}

#[derive(Debug)]
pub struct PluginRuntime {
    catalog_dir: PathBuf,
    package_root: PathBuf,
    catalog: BTreeMap<String, CatalogPlugin>,
    installed: BTreeMap<String, InstalledPlugin>,
    instances: BTreeMap<String, PluginInstance>,
}

impl PluginRuntime {
    pub fn new(
        catalog_dir: impl Into<PathBuf>,
        package_root: impl Into<PathBuf>,
    ) -> Result<Self, RuntimeError> {
        let catalog_dir = catalog_dir.into();
        let package_root = package_root.into();
        fs::create_dir_all(&catalog_dir)?;
        fs::create_dir_all(&package_root)?;

        let mut runtime = Self {
            catalog_dir,
            package_root,
            catalog: BTreeMap::new(),
            installed: BTreeMap::new(),
            instances: BTreeMap::new(),
        };
        runtime.refresh_catalog()?;
        Ok(runtime)
    }

    pub fn catalog_dir(&self) -> &Path {
        &self.catalog_dir
    }

    pub fn package_root(&self) -> &Path {
        &self.package_root
    }

    pub fn refresh_catalog(&mut self) -> Result<(), RuntimeError> {
        self.catalog.clear();
        for entry in fs::read_dir(&self.catalog_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("azplugin") {
                continue;
            }
            let manifest = read_manifest_from_package(&path)?;
            self.catalog.insert(
                manifest.descriptor.id.clone(),
                CatalogPlugin {
                    manifest,
                    package_path: path,
                },
            );
        }
        Ok(())
    }

    pub fn installed_descriptors(&self) -> Vec<PluginDescriptor> {
        self.installed
            .values()
            .map(|plugin| plugin.manifest.descriptor.clone())
            .collect()
    }

    pub fn instances(&self) -> Vec<PluginInstance> {
        self.instances.values().cloned().collect()
    }

    pub fn marketplace_snapshot(&self) -> MarketplaceSnapshot {
        let mut entries = Vec::new();
        let mut tags = BTreeSet::new();
        for plugin in self.catalog.values() {
            for tag in &plugin.manifest.descriptor.tags {
                tags.insert(tag.clone());
            }
            let instances = self
                .instances
                .values()
                .filter(|instance| instance.plugin_id == plugin.manifest.descriptor.id)
                .count();
            entries.push(MarketplaceEntry {
                plugin_id: plugin.manifest.descriptor.id.clone(),
                name: plugin.manifest.descriptor.name.clone(),
                version: plugin.manifest.descriptor.version.clone(),
                kind: plugin.manifest.descriptor.kind.clone(),
                summary: plugin.manifest.descriptor.summary.clone(),
                tags: plugin.manifest.descriptor.tags.clone(),
                icon: plugin.manifest.descriptor.icon.clone(),
                compatibility: plugin.manifest.descriptor.compatibility.clone(),
                capabilities: plugin.manifest.descriptor.capabilities.clone(),
                status: if self.installed.contains_key(&plugin.manifest.descriptor.id) {
                    PluginStatus::Installed
                } else {
                    PluginStatus::Available
                },
                instances,
            });
        }
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        MarketplaceSnapshot {
            entries,
            tags: tags.into_iter().collect(),
        }
    }

    pub fn install_from_catalog(
        &mut self,
        plugin_id: &str,
    ) -> Result<PluginDescriptor, RuntimeError> {
        let Some(catalog) = self.catalog.get(plugin_id) else {
            return Err(RuntimeError::PackageNotFound(plugin_id.to_string()));
        };
        if self.installed.contains_key(plugin_id) {
            return Err(RuntimeError::AlreadyInstalled(plugin_id.to_string()));
        }
        validate_package(&catalog.package_path)?;

        let install_dir = self
            .package_root
            .join(plugin_id)
            .join(&catalog.manifest.descriptor.version);
        if install_dir.exists() {
            fs::remove_dir_all(&install_dir)?;
        }
        fs::create_dir_all(&install_dir)?;
        unpack_package(&catalog.package_path, &install_dir)?;

        let installed = InstalledPlugin {
            manifest: catalog.manifest.clone(),
            install_dir,
        };
        let descriptor = installed.manifest.descriptor.clone();
        self.installed.insert(plugin_id.to_string(), installed);
        Ok(descriptor)
    }

    pub fn create_instance(
        &mut self,
        plugin_id: &str,
        label: &str,
    ) -> Result<PluginInstance, RuntimeError> {
        let Some(plugin) = self.installed.get(plugin_id) else {
            return Err(RuntimeError::NotInstalled(plugin_id.to_string()));
        };
        let slug_base = slugify(label);
        let slug = unique_slug(slug_base, self.instances.keys());
        let instance = PluginInstance {
            plugin_id: plugin_id.to_string(),
            plugin_name: plugin.manifest.descriptor.name.clone(),
            slug: slug.clone(),
            label: label.to_string(),
            status: PluginStatus::Installed,
            page_ids: plugin
                .manifest
                .descriptor
                .pages
                .iter()
                .map(|page| page.id.clone())
                .collect(),
            tags: plugin.manifest.descriptor.tags.clone(),
            created_at: Utc::now(),
            config: PluginInstanceConfig {
                label: label.to_string(),
                permissions: vec![format!("plugin:{plugin_id}:instance:{slug}:read")],
                dictionary_namespace: Some(format!("{plugin_id}.{slug}")),
                allowed_origins: vec![],
            },
        };
        self.instances.insert(slug, instance.clone());
        Ok(instance)
    }

    pub fn ensure_dev_package(
        &self,
        source_dir: &Path,
        package_name: &str,
    ) -> Result<PathBuf, RuntimeError> {
        let package_path = self.catalog_dir.join(format!("{package_name}.azplugin"));
        create_package_from_dir(source_dir, &package_path)?;
        Ok(package_path)
    }
}

pub fn read_manifest_from_package(path: &Path) -> Result<PluginPackageManifest, RuntimeError> {
    let file = fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut manifest = archive
        .by_name("plugin.toml")
        .map_err(|_| RuntimeError::InvalidPackage("plugin.toml is required".to_string()))?;
    let mut source = String::new();
    manifest.read_to_string(&mut source)?;
    let manifest: PluginPackageManifest = toml_edit::de::from_str(&source)?;
    Ok(manifest)
}

pub fn validate_package(path: &Path) -> Result<(), RuntimeError> {
    let file = fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut content = String::new();
    {
        let mut checksum_file = archive.by_name("checksums.sha256").map_err(|_| {
            RuntimeError::InvalidPackage("checksums.sha256 is required".to_string())
        })?;
        checksum_file.read_to_string(&mut content)?;
    }

    let checksums = parse_checksums(&content);
    if checksums.is_empty() {
        return Err(RuntimeError::InvalidPackage(
            "checksums.sha256 did not contain any entries".to_string(),
        ));
    }

    for (entry_path, expected) in checksums {
        let mut entry = archive.by_name(&entry_path).map_err(|_| {
            RuntimeError::InvalidPackage(format!("missing packaged file `{entry_path}`"))
        })?;
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        let actual = sha256_hex(&bytes);
        if actual != expected {
            return Err(RuntimeError::InvalidPackage(format!(
                "checksum mismatch for `{entry_path}`"
            )));
        }
    }

    let manifest = read_manifest_from_package(path)?;
    if manifest.runtime.binary_path.is_empty() {
        return Err(RuntimeError::InvalidPackage(
            "runtime.binary_path cannot be empty".to_string(),
        ));
    }
    Ok(())
}

pub fn unpack_package(path: &Path, target_dir: &Path) -> Result<(), RuntimeError> {
    let file = fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let out_path = target_dir.join(entry.name());
        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output = fs::File::create(out_path)?;
        std::io::copy(&mut entry, &mut output)?;
    }
    Ok(())
}

pub fn create_package_from_dir(source_dir: &Path, output_path: &Path) -> Result<(), RuntimeError> {
    let file = fs::File::create(output_path)?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default();
    let entries = package_entries(source_dir)?;
    for relative_path in entries {
        let disk_path = source_dir.join(&relative_path);
        if disk_path.is_dir() {
            writer.add_directory(relative_path.to_string_lossy(), options)?;
        } else {
            writer.start_file(relative_path.to_string_lossy(), options)?;
            let bytes = fs::read(disk_path)?;
            writer.write_all(&bytes)?;
        }
    }
    writer.finish()?;
    Ok(())
}

fn package_entries(source_dir: &Path) -> Result<Vec<PathBuf>, RuntimeError> {
    let mut entries = Vec::new();
    walk(source_dir, source_dir, &mut entries)?;
    entries.sort();
    Ok(entries)
}

fn walk(root: &Path, dir: &Path, entries: &mut Vec<PathBuf>) -> Result<(), RuntimeError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .expect("relative path should strip")
            .to_path_buf();
        if path.is_dir() {
            entries.push(relative.clone());
            walk(root, &path, entries)?;
        } else {
            entries.push(relative);
        }
    }
    Ok(())
}

fn parse_checksums(content: &str) -> BTreeMap<String, String> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let (hash, path) = trimmed.split_once("  ")?;
            Some((path.to_string(), hash.to_string()))
        })
        .collect()
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in value.chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            last_dash = false;
            Some(ch.to_ascii_lowercase())
        } else if !last_dash {
            last_dash = true;
            Some('-')
        } else {
            None
        };
        if let Some(ch) = mapped {
            slug.push(ch);
        }
    }
    slug.trim_matches('-').to_string()
}

fn unique_slug<'a>(base: String, existing: impl Iterator<Item = &'a String>) -> String {
    let mut candidate = if base.is_empty() {
        "plugin-instance".to_string()
    } else {
        base
    };
    let existing: BTreeSet<_> = existing.cloned().collect();
    if !existing.contains(&candidate) {
        return candidate;
    }
    let suffix = Uuid::new_v4()
        .simple()
        .to_string()
        .chars()
        .take(6)
        .collect::<String>();
    candidate.push('-');
    candidate.push_str(&suffix);
    candidate
}
