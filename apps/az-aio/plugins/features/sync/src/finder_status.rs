use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    error::{SyncError, SyncResult},
    sync_model::{SyncDocumentRecord, SyncFileStatus, SyncRoot},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FinderBadge {
    Hosted,
    Shared,
    Busy,
    Error,
    None,
}

impl FinderBadge {
    pub fn id(self) -> &'static str {
        match self {
            Self::Hosted => "hosted",
            Self::Shared => "shared",
            Self::Busy => "busy",
            Self::Error => "error",
            Self::None => "",
        }
    }

    pub fn for_status(status: SyncFileStatus) -> Self {
        match status {
            SyncFileStatus::Synced => Self::Hosted,
            SyncFileStatus::Shared => Self::Shared,
            SyncFileStatus::Syncing => Self::Busy,
            SyncFileStatus::Error => Self::Error,
            SyncFileStatus::Deleted => Self::None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FinderSyncState {
    #[serde(default)]
    pub hosted: Vec<FinderHostedItem>,
    #[serde(default)]
    pub hosted_roots: Vec<FinderHostedItem>,
}

impl FinderSyncState {
    pub fn from_roots_and_files(roots: &[SyncRoot], files: &[SyncDocumentRecord]) -> Self {
        Self {
            hosted_roots: roots.iter().map(FinderHostedItem::from_root).collect(),
            hosted: files
                .iter()
                .filter(|file| file.status != SyncFileStatus::Deleted)
                .map(FinderHostedItem::from_file)
                .collect(),
        }
    }

    pub fn to_pretty_json(&self) -> SyncResult<String> {
        serde_json::to_string_pretty(self).map_err(|source| SyncError::Json {
            path: PathBuf::from("<memory>"),
            source,
        })
    }

    pub fn write_to_path(&self, path: impl AsRef<Path>) -> SyncResult<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| SyncError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let json = self.to_pretty_json()?;
        fs::write(path, json).map_err(|source| SyncError::Io {
            path: path.to_path_buf(),
            source,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FinderHostedItem {
    pub local_path: String,
    pub space_id: String,
    pub root_alias: String,
    pub relative_path: String,
    pub badge: String,
}

impl FinderHostedItem {
    pub fn from_root(root: &SyncRoot) -> Self {
        Self {
            local_path: root.local_path.to_string_lossy().to_string(),
            space_id: root.space_id.clone(),
            root_alias: root.alias.clone(),
            relative_path: root.relative_path.clone(),
            badge: FinderBadge::Hosted.id().to_string(),
        }
    }

    pub fn from_file(file: &SyncDocumentRecord) -> Self {
        Self {
            local_path: file.local_path.to_string_lossy().to_string(),
            space_id: if file.status == SyncFileStatus::Shared {
                "shared".to_string()
            } else {
                "main".to_string()
            },
            root_alias: "default".to_string(),
            relative_path: file.relative_path.clone(),
            badge: FinderBadge::for_status(file.status).id().to_string(),
        }
    }
}

pub fn default_finder_state_path(home_dir: impl AsRef<Path>) -> PathBuf {
    home_dir
        .as_ref()
        .join("Library")
        .join("Application Support")
        .join("addzero")
        .join("drive")
        .join("state.json")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::sync_model::{SyncDeviceInfo, SyncRoot};

    use super::{FinderBadge, FinderSyncState};

    #[test]
    fn finder_state_matches_existing_extension_schema() -> Result<(), Box<dyn std::error::Error>> {
        let device = SyncDeviceInfo::new("mac-a", PathBuf::from("/tmp/mac-a"));
        let root = SyncRoot::default_for_device(&device);
        let mut engine = crate::SyncEngine::with_device(device);
        engine.apply_local_text("/tmp/mac-a/az-sync/a.txt", "alpha\nbeta")?;

        let state = FinderSyncState::from_roots_and_files(&[root], &engine.files());
        let json = state.to_pretty_json()?;
        assert!(json.contains("\"hosted\""));
        assert!(json.contains("\"hosted_roots\""));
        assert!(json.contains("\"local_path\""));
        assert_eq!(
            FinderBadge::for_status(crate::SyncFileStatus::Synced).id(),
            "hosted"
        );
        Ok(())
    }
}
