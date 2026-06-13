#![cfg(not(target_arch = "wasm32"))]

use std::{
    collections::VecDeque,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    error::{SyncError, SyncResult},
    finder_status::default_finder_state_path,
    sync_engine::SyncEngine,
    sync_index::{SyncIndexSummary, default_local_index_path},
    sync_model::{SyncDeviceInfo, SyncDocumentRecord, SyncRoot, normalize_home_relative_path},
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncAgentConfig {
    pub device: SyncDeviceInfo,
    pub roots: Vec<SyncAgentRoot>,
    pub write_local_index: bool,
    pub write_finder_state: bool,
}

impl SyncAgentConfig {
    pub fn for_device(device: SyncDeviceInfo) -> Self {
        Self {
            device,
            roots: Vec::new(),
            write_local_index: true,
            write_finder_state: true,
        }
    }

    pub fn detected() -> Self {
        Self::for_device(SyncDeviceInfo::detect())
    }

    pub fn with_root(mut self, root: SyncAgentRoot) -> Self {
        self.roots.push(root);
        self
    }

    pub fn merge_persisted_roots(mut self) -> SyncResult<Self> {
        let stored = SyncAgentRootsConfig::read_from_default_path(&self.device.home_dir)?;
        for root in stored.roots {
            if !self
                .roots
                .iter()
                .any(|existing| existing.relative_path == root.relative_path)
            {
                self.roots.push(root);
            }
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncAgentRoot {
    pub alias: String,
    pub relative_path: String,
    pub space_id: String,
}

impl SyncAgentRoot {
    pub fn new(
        alias: impl Into<String>,
        relative_path: impl AsRef<str>,
        space_id: impl Into<String>,
    ) -> SyncResult<Self> {
        Ok(Self {
            alias: alias.into(),
            relative_path: normalize_home_relative_path(relative_path.as_ref())?,
            space_id: space_id.into(),
        })
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncAgentRootsConfig {
    pub roots: Vec<SyncAgentRoot>,
}

impl SyncAgentRootsConfig {
    pub fn read_from_default_path(home_dir: impl AsRef<Path>) -> SyncResult<Self> {
        Self::read_from_path(default_roots_config_path(home_dir))
    }

    pub fn read_from_path(path: impl AsRef<Path>) -> SyncResult<Self> {
        let path = path.as_ref();
        match fs::read_to_string(path) {
            Ok(json) => serde_json::from_str(&json).map_err(|source| SyncError::Json {
                path: path.to_path_buf(),
                source,
            }),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(source) => Err(SyncError::Io {
                path: path.to_path_buf(),
                source,
            }),
        }
    }

    pub fn write_to_default_path(&self, home_dir: impl AsRef<Path>) -> SyncResult<()> {
        self.write_to_path(default_roots_config_path(home_dir))
    }

    pub fn write_to_path(&self, path: impl AsRef<Path>) -> SyncResult<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| SyncError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|source| SyncError::Json {
            path: path.to_path_buf(),
            source,
        })?;
        fs::write(path, json).map_err(|source| SyncError::Io {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn upsert_root(&mut self, root: SyncAgentRoot) {
        if let Some(existing) = self
            .roots
            .iter_mut()
            .find(|existing| existing.relative_path == root.relative_path)
        {
            *existing = root;
        } else {
            self.roots.push(root);
        }
        self.roots.sort_by(|left, right| {
            left.relative_path
                .cmp(&right.relative_path)
                .then_with(|| left.alias.cmp(&right.alias))
        });
    }
}

pub fn default_roots_config_path(home_dir: impl AsRef<Path>) -> PathBuf {
    home_dir
        .as_ref()
        .join(".config")
        .join("addzero")
        .join("sync")
        .join("roots.json")
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SyncAgentBootstrapReport {
    pub device: SyncDeviceInfo,
    pub roots: Vec<SyncRoot>,
    pub imported_files: Vec<SyncDocumentRecord>,
    pub local_index: SyncIndexSummary,
    pub local_index_path: PathBuf,
    pub finder_state_path: PathBuf,
}

pub fn bootstrap_sync_agent(config: SyncAgentConfig) -> SyncResult<SyncAgentBootstrapReport> {
    let engine = build_sync_agent_engine(&config)?;
    if config.write_local_index {
        engine.write_default_local_index()?;
    }
    if config.write_finder_state {
        engine.write_default_finder_state()?;
    }

    let local_index = engine.local_index().summary();
    Ok(SyncAgentBootstrapReport {
        device: config.device.clone(),
        roots: engine.roots(),
        imported_files: engine.files(),
        local_index,
        local_index_path: default_local_index_path(&config.device.home_dir),
        finder_state_path: default_finder_state_path(&config.device.home_dir),
    })
}

pub fn build_sync_agent_engine(config: &SyncAgentConfig) -> SyncResult<SyncEngine> {
    let mut engine = SyncEngine::with_device(config.device.clone());
    for root in &config.roots {
        engine.add_root(&root.alias, &root.relative_path, &root.space_id)?;
    }

    for root in engine.roots() {
        fs::create_dir_all(&root.local_path).map_err(|source| SyncError::Io {
            path: root.local_path.clone(),
            source,
        })?;
        import_existing_text_files(&mut engine, &root)?;
    }

    Ok(engine)
}

fn import_existing_text_files(engine: &mut SyncEngine, root: &SyncRoot) -> SyncResult<()> {
    let mut pending = VecDeque::from([root.local_path.clone()]);
    while let Some(path) = pending.pop_front() {
        let entries = fs::read_dir(&path).map_err(|source| SyncError::Io {
            path: path.clone(),
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| SyncError::Io {
                path: path.clone(),
                source,
            })?;
            let path = entry.path();
            let metadata = entry.metadata().map_err(|source| SyncError::Io {
                path: path.clone(),
                source,
            })?;
            if metadata.is_dir() {
                pending.push_back(path);
                continue;
            }
            if !metadata.is_file() {
                continue;
            }
            import_text_file_if_utf8(engine, &path)?;
        }
    }
    Ok(())
}

fn import_text_file_if_utf8(engine: &mut SyncEngine, path: &Path) -> SyncResult<()> {
    let bytes = fs::read(path).map_err(|source| SyncError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let Ok(text) = String::from_utf8(bytes) else {
        return Ok(());
    };
    engine.apply_local_text(path, &text)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{
        SyncLocalIndex,
        sync_agent::{SyncAgentConfig, SyncAgentRoot, bootstrap_sync_agent},
        sync_agent::{SyncAgentRootsConfig, default_roots_config_path},
        sync_index::default_local_index_path,
        sync_model::SyncDeviceInfo,
    };

    #[test]
    fn bootstrap_creates_default_root_and_persists_index() -> Result<(), Box<dyn std::error::Error>>
    {
        let temp_dir = tempfile::tempdir()?;
        let home_dir = temp_dir.path().join("home-a");
        let config = SyncAgentConfig::for_device(SyncDeviceInfo::new("mac-a", home_dir.clone()));

        let report = bootstrap_sync_agent(config)?;
        let restored = SyncLocalIndex::read_from_path(default_local_index_path(&home_dir))?;

        assert!(home_dir.join("az-sync").is_dir());
        assert_eq!(report.local_index.file_count, 0);
        assert_eq!(restored.roots[0].relative_path, "az-sync");
        Ok(())
    }

    #[test]
    fn bootstrap_imports_existing_home_relative_roots() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let home_dir = temp_dir.path().join("home-b");
        let skills_dir = home_dir.join(".agents/skills/demo");
        fs::create_dir_all(&skills_dir)?;
        fs::write(skills_dir.join("SKILL.md"), "one\ntwo\nthree")?;
        fs::write(skills_dir.join("binary.bin"), [0, 159, 146, 150])?;
        let config = SyncAgentConfig::for_device(SyncDeviceInfo::new("mac-b", home_dir.clone()))
            .with_root(SyncAgentRoot::new("skills", ".agents/skills", "main")?);

        let report = bootstrap_sync_agent(config)?;
        let restored = SyncLocalIndex::read_from_path(default_local_index_path(&home_dir))?;

        assert!(restored.files.contains_key(".agents/skills/demo/SKILL.md"));
        assert!(
            !restored
                .files
                .contains_key(".agents/skills/demo/binary.bin")
        );
        assert_eq!(report.local_index.file_count, 1);
        Ok(())
    }

    #[test]
    fn bootstrap_keeps_paths_home_relative_across_devices() -> Result<(), Box<dyn std::error::Error>>
    {
        let temp_dir = tempfile::tempdir()?;
        let home_a = temp_dir.path().join("a-home");
        let home_b = temp_dir.path().join("b-home");
        fs::create_dir_all(home_a.join("az-sync"))?;
        fs::create_dir_all(home_b.join("az-sync"))?;
        fs::write(home_a.join("az-sync/a.txt"), "alpha")?;
        fs::write(home_b.join("az-sync/a.txt"), "alpha")?;

        let report_a = bootstrap_sync_agent(SyncAgentConfig::for_device(SyncDeviceInfo::new(
            "a", home_a,
        )))?;
        let report_b = bootstrap_sync_agent(SyncAgentConfig::for_device(SyncDeviceInfo::new(
            "b", home_b,
        )))?;

        assert_eq!(
            report_a.imported_files[0].relative_path,
            report_b.imported_files[0].relative_path
        );
        assert_ne!(
            report_a.imported_files[0].local_path,
            report_b.imported_files[0].local_path
        );
        Ok(())
    }

    #[test]
    fn roots_config_round_trips_and_merges_into_agent_config()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let home_dir = temp_dir.path().join("home-c");
        let mut roots = SyncAgentRootsConfig::default();
        roots.upsert_root(SyncAgentRoot::new("skills", ".agents/skills", "skills")?);
        roots.write_to_default_path(&home_dir)?;

        let restored = SyncAgentRootsConfig::read_from_path(default_roots_config_path(&home_dir))?;
        let config = SyncAgentConfig::for_device(SyncDeviceInfo::new("mac-c", home_dir))
            .merge_persisted_roots()?;

        assert_eq!(restored.roots[0].relative_path, ".agents/skills");
        assert_eq!(config.roots[0].space_id, "skills");
        Ok(())
    }
}
