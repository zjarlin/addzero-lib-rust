use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result};
use az_crdt::document::LineCrdtDocument;
use az_crdt::wire::{LineCrdtUpdate, LineCrdtVersion};

use crate::{
    contracts::{SyncStatusResponse, SyncTransportSummary},
    finder_status::{FinderSyncState, default_finder_state_path},
    sync_index::{SyncLocalIndex, default_local_index_path},
    sync_model::{
        SyncBlobKind, SyncCrdtEnvelope, SyncDeviceInfo, SyncDocumentRecord, SyncFileStatus,
        SyncRoot, content_hash, normalize_home_relative_path,
    },
};

pub struct SyncEngine {
    device: SyncDeviceInfo,
    roots: BTreeMap<String, SyncRoot>,
    documents: BTreeMap<String, SyncDocumentEntry>,
}

impl SyncEngine {
    pub fn new() -> Self {
        Self::with_device(SyncDeviceInfo::detect())
    }

    pub fn with_device(device: SyncDeviceInfo) -> Self {
        let default_root = SyncRoot::default_for_device(&device);
        Self {
            device,
            roots: BTreeMap::from([(default_root.alias.clone(), default_root)]),
            documents: BTreeMap::new(),
        }
    }

    pub fn device(&self) -> &SyncDeviceInfo {
        &self.device
    }

    pub fn roots(&self) -> Vec<SyncRoot> {
        self.roots.values().cloned().collect()
    }

    pub fn add_root(
        &mut self,
        alias: impl Into<String>,
        relative_path: &str,
        space_id: impl Into<String>,
    ) -> Result<SyncRoot> {
        let root = SyncRoot::from_home_relative(&self.device, alias, relative_path, space_id)?;
        self.roots.insert(root.alias.clone(), root.clone());
        Ok(root)
    }

    pub fn files(&self) -> Vec<SyncDocumentRecord> {
        self.documents
            .values()
            .map(|entry| self.record_for_entry(entry))
            .collect()
    }

    pub fn status(&self) -> SyncStatusResponse {
        SyncStatusResponse {
            device: self.device.clone(),
            connected_devices: vec![self.device.clone()],
            default_root: self
                .device
                .default_sync_root()
                .to_string_lossy()
                .to_string(),
            roots: self.roots(),
            files: self.files(),
            local_index: self.local_index().summary(),
            websocket: SyncTransportSummary::default(),
            finder_state_path: Some(
                default_finder_state_path(&self.device.home_dir)
                    .to_string_lossy()
                    .to_string(),
            ),
        }
    }

    pub fn local_index(&self) -> SyncLocalIndex {
        let mut index = SyncLocalIndex::new(self.device.clone(), self.roots());
        for file in self.files() {
            if let Err(_error) = index.upsert_document(&file) {
                let _ = index.upsert_document_with_metadata(
                    &file,
                    crate::sync_index::SyncFileMetadata::missing(),
                );
            }
        }
        index
    }

    pub fn write_default_local_index(&self) -> Result<()> {
        self.local_index()
            .write_to_path(default_local_index_path(&self.device.home_dir))
    }

    pub fn apply_local_text(
        &mut self,
        local_path: impl AsRef<Path>,
        text: &str,
    ) -> Result<SyncDocumentRecord> {
        let relative_path = self.device.home_relative_path(local_path.as_ref())?;
        let entry = self.entry_for_relative_path(&relative_path)?;
        entry
            .document
            .apply_text_by_line(text)
            .context("CRDT apply local text by line failed")?;
        entry.status = SyncFileStatus::Syncing;
        Ok(self.record_for_relative_path(&relative_path)?)
    }

    pub fn apply_precise_text(
        &mut self,
        local_path: impl AsRef<Path>,
        text: &str,
    ) -> Result<SyncDocumentRecord> {
        let relative_path = self.device.home_relative_path(local_path.as_ref())?;
        let entry = self.entry_for_relative_path(&relative_path)?;
        entry
            .document
            .apply_text_precise(text)
            .context("CRDT apply local precise text failed")?;
        entry.status = SyncFileStatus::Syncing;
        Ok(self.record_for_relative_path(&relative_path)?)
    }

    pub fn replace_line(
        &mut self,
        relative_path: &str,
        line_index: usize,
        line: &str,
    ) -> Result<SyncDocumentRecord> {
        let relative_path = normalize_home_relative_path(relative_path)?;
        let entry = self.entry_for_relative_path(&relative_path)?;
        entry
            .document
            .replace_line(line_index, line)
            .context("CRDT replace line failed")?;
        entry.status = SyncFileStatus::Syncing;
        Ok(self.record_for_relative_path(&relative_path)?)
    }

    pub fn delete_text(
        &mut self,
        relative_path: &str,
        unicode_index: usize,
        unicode_len: usize,
    ) -> Result<SyncDocumentRecord> {
        let relative_path = normalize_home_relative_path(relative_path)?;
        let entry = self.entry_for_relative_path(&relative_path)?;
        entry
            .document
            .delete_text(unicode_index, unicode_len)
            .context("CRDT delete text failed")?;
        entry.status = SyncFileStatus::Syncing;
        Ok(self.record_for_relative_path(&relative_path)?)
    }

    pub fn delete_file(&mut self, relative_path: &str) -> Result<SyncDocumentRecord> {
        let relative_path = normalize_home_relative_path(relative_path)?;
        let entry = self.entry_for_relative_path(&relative_path)?;
        entry
            .document
            .apply_text_by_line("")
            .context("CRDT mark file deleted failed")?;
        entry.status = SyncFileStatus::Deleted;
        Ok(self.record_for_relative_path(&relative_path)?)
    }

    pub fn materialize_text(&self, relative_path: &str) -> Result<String> {
        let relative_path = normalize_home_relative_path(relative_path)?;
        let entry =
            self.documents
                .get(&relative_path)
                .with_context(|| format!("sync document `{relative_path}` does not exist"))?;
        Ok(entry.document.text())
    }

    pub fn materialize_text_to_local_file(
        &self,
        relative_path: &str,
    ) -> Result<std::path::PathBuf> {
        let relative_path = normalize_home_relative_path(relative_path)?;
        let local_path = self.device.local_path_for_home_relative(&relative_path)?;
        if let Some(parent) = local_path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("I/O failed for `{parent:?}`"))?;
        }
        let text = self.materialize_text(&relative_path)?;
        fs::write(&local_path, text).with_context(|| format!("I/O failed for `{local_path:?}`"))?;
        Ok(local_path)
    }

    pub fn export_snapshot(&self, relative_path: &str) -> Result<SyncCrdtEnvelope> {
        let relative_path = normalize_home_relative_path(relative_path)?;
        let entry =
            self.documents
                .get(&relative_path)
                .with_context(|| format!("sync document `{relative_path}` does not exist"))?;
        let snapshot = entry
            .document
            .export_snapshot()
            .context("CRDT export snapshot failed")?;
        Ok(SyncCrdtEnvelope {
            relative_path,
            source_device: self.device.device_name.clone(),
            base_version: None,
            version: entry.document.version().into_bytes(),
            kind: SyncBlobKind::Snapshot,
            blob: snapshot.into_bytes(),
            content_hash: content_hash(&entry.document.text()),
        })
    }

    pub fn export_update_since(
        &self,
        relative_path: &str,
        remote_version: Option<&LineCrdtVersion>,
    ) -> Result<SyncCrdtEnvelope> {
        let relative_path = normalize_home_relative_path(relative_path)?;
        let entry =
            self.documents
                .get(&relative_path)
                .with_context(|| format!("sync document `{relative_path}` does not exist"))?;
        let update = match remote_version {
            Some(version) => entry
                .document
                .export_updates_since(version)
                .context("CRDT export incremental update failed")?,
            None => entry
                .document
                .export_all_updates()
                .context("CRDT export full update stream failed")?,
        };
        Ok(SyncCrdtEnvelope {
            relative_path,
            source_device: self.device.device_name.clone(),
            base_version: remote_version.map(|version| version.clone().into_bytes()),
            version: entry.document.version().into_bytes(),
            kind: SyncBlobKind::Update,
            blob: update.into_bytes(),
            content_hash: content_hash(&entry.document.text()),
        })
    }

    pub fn import_remote_blob(
        &mut self,
        envelope: SyncCrdtEnvelope,
    ) -> Result<SyncDocumentRecord> {
        let relative_path = normalize_home_relative_path(&envelope.relative_path)?;
        if !self.documents.contains_key(&relative_path) {
            let document =
                LineCrdtDocument::with_peer_id(self.device.peer_id_for_path(&relative_path))
                    .context("CRDT create document failed")?;
            self.documents.insert(
                relative_path.clone(),
                SyncDocumentEntry {
                    relative_path: relative_path.clone(),
                    local_path: self.device.local_path_for_home_relative(&relative_path)?,
                    document,
                    status: SyncFileStatus::Synced,
                },
            );
        }
        let entry =
            self.documents
                .get_mut(&relative_path)
                .with_context(|| format!("sync document `{relative_path}` does not exist"))?;
        match envelope.kind {
            SyncBlobKind::Snapshot => {
                entry
                    .document
                    .import_snapshot(envelope.blob)
                    .context("CRDT import snapshot failed")?;
            }
            SyncBlobKind::Update => {
                let update = LineCrdtUpdate::from_bytes(envelope.blob);
                entry
                    .document
                    .import_update(update)
                    .context("CRDT import update failed")?;
            }
        }
        entry.status = SyncFileStatus::Synced;
        Ok(self.record_for_relative_path(&relative_path)?)
    }

    pub fn finder_state(&self) -> FinderSyncState {
        FinderSyncState::from_roots_and_files(&self.roots(), &self.files())
    }

    pub fn write_default_finder_state(&self) -> Result<()> {
        self.finder_state()
            .write_to_path(default_finder_state_path(&self.device.home_dir))
    }

    fn entry_for_relative_path(
        &mut self,
        relative_path: &str,
    ) -> Result<&mut SyncDocumentEntry> {
        let relative_path = normalize_home_relative_path(relative_path)?;
        if !self.documents.contains_key(&relative_path) {
            let document =
                LineCrdtDocument::with_peer_id(self.device.peer_id_for_path(&relative_path))
                    .context("CRDT create document failed")?;
            self.documents.insert(
                relative_path.clone(),
                SyncDocumentEntry {
                    relative_path: relative_path.clone(),
                    local_path: self.device.local_path_for_home_relative(&relative_path)?,
                    document,
                    status: SyncFileStatus::Synced,
                },
            );
        }
        self.documents
            .get_mut(&relative_path)
            .with_context(|| format!("sync document `{relative_path}` does not exist"))
    }

    fn record_for_relative_path(&self, relative_path: &str) -> Result<SyncDocumentRecord> {
        let entry =
            self.documents
                .get(relative_path)
                .with_context(|| format!("sync document `{relative_path}` does not exist"))?;
        Ok(self.record_for_entry(entry))
    }

    fn record_for_entry(&self, entry: &SyncDocumentEntry) -> SyncDocumentRecord {
        SyncDocumentRecord {
            relative_path: entry.relative_path.clone(),
            local_path: entry.local_path.clone(),
            device_name: self.device.device_name.clone(),
            home_dir: self.device.home_dir.clone(),
            crdt_snapshot: entry
                .document
                .export_snapshot()
                .map(|snapshot| snapshot.into_bytes())
                .unwrap_or_default(),
            crdt_version: entry.document.version().into_bytes(),
            content_hash: content_hash(&entry.document.text()),
            status: entry.status,
        }
    }
}

impl Default for SyncEngine {
    fn default() -> Self {
        Self::new()
    }
}

struct SyncDocumentEntry {
    relative_path: String,
    local_path: std::path::PathBuf,
    document: LineCrdtDocument,
    status: SyncFileStatus,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use az_crdt::wire::LineCrdtVersion;

    use crate::sync_model::SyncDeviceInfo;

    use super::SyncEngine;

    #[test]
    fn engines_converge_after_line_and_precise_edits() -> Result<(), Box<dyn std::error::Error>> {
        let mut left =
            SyncEngine::with_device(SyncDeviceInfo::new("mac-left", PathBuf::from("/tmp/left")));
        let mut right = SyncEngine::with_device(SyncDeviceInfo::new(
            "mac-right",
            PathBuf::from("/tmp/right"),
        ));
        let left_path = PathBuf::from("/tmp/left/az-sync/note.txt");
        let right_path = PathBuf::from("/tmp/right/az-sync/note.txt");

        let left_record = left.apply_local_text(&left_path, "one\ntwo\nthree")?;
        let first_update = left.export_update_since("az-sync/note.txt", None)?;
        right.import_remote_blob(first_update)?;
        assert_eq!(
            right.materialize_text("az-sync/note.txt")?,
            "one\ntwo\nthree"
        );

        right.delete_text("az-sync/note.txt", 4, 3)?;
        let left_version = LineCrdtVersion::from_bytes(left_record.crdt_version);
        let second_update = right.export_update_since("az-sync/note.txt", Some(&left_version))?;
        left.import_remote_blob(second_update)?;

        assert_eq!(left.materialize_text("az-sync/note.txt")?, "one\n\nthree");
        assert_eq!(
            left.materialize_text("az-sync/note.txt")?,
            right.materialize_text("az-sync/note.txt")?
        );

        left.apply_local_text(&left_path, "one\nlocal\nthree")?;
        right.apply_local_text(&right_path, "one\nremote\nthree")?;
        let left_delta = left.export_update_since("az-sync/note.txt", None)?;
        let right_delta = right.export_update_since("az-sync/note.txt", None)?;
        left.import_remote_blob(right_delta)?;
        right.import_remote_blob(left_delta)?;
        assert_eq!(
            left.materialize_text("az-sync/note.txt")?,
            right.materialize_text("az-sync/note.txt")?
        );
        Ok(())
    }

    #[test]
    fn status_exposes_default_home_sync_root() -> Result<(), Box<dyn std::error::Error>> {
        let mut engine =
            SyncEngine::with_device(SyncDeviceInfo::new("mac-a", PathBuf::from("/tmp/mac-a")));
        engine.apply_local_text("/tmp/mac-a/az-sync/a.txt", "alpha")?;

        let status = engine.status();
        assert_eq!(status.roots[0].relative_path, "az-sync");
        assert_eq!(status.files[0].relative_path, "az-sync/a.txt");
        assert_eq!(status.local_index.file_count, 1);
        assert!(status.local_index.stored_outside_sync_roots);
        assert_eq!(status.websocket.endpoint, "/api/sync/ws");
        Ok(())
    }

    #[test]
    fn delete_file_marks_index_deleted_and_removes_finder_hosted_entry()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut engine =
            SyncEngine::with_device(SyncDeviceInfo::new("mac-a", PathBuf::from("/tmp/mac-a")));
        engine.apply_local_text("/tmp/mac-a/az-sync/a.txt", "alpha")?;

        let deleted = engine.delete_file("az-sync/a.txt")?;
        let status = engine.status();
        let finder_state = engine.finder_state();

        assert_eq!(deleted.status, crate::SyncFileStatus::Deleted);
        assert_eq!(status.local_index.deleted_count, 1);
        assert!(finder_state.hosted.is_empty());
        Ok(())
    }
}
