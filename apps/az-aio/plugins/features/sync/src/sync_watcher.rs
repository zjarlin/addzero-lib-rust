#![cfg(not(target_arch = "wasm32"))]

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    time::Duration,
};

use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
    event::{ModifyKind, RenameMode},
};

use crate::{
    error::{SyncError, SyncResult},
    sync_engine::SyncEngine,
    sync_model::{SyncDeviceInfo, SyncDocumentRecord, SyncRoot, normalize_home_relative_path},
    sync_server::SyncObjectManifest,
};

pub const DEFAULT_WATCH_DEBOUNCE_MS: u64 = 250;

#[derive(Debug)]
pub struct SyncRootWatcher {
    _watcher: RecommendedWatcher,
    roots: Vec<SyncRoot>,
}

impl SyncRootWatcher {
    pub fn watch_roots(
        roots: Vec<SyncRoot>,
        mut on_event: impl FnMut(SyncWatchEvent) + Send + 'static,
    ) -> SyncResult<Self> {
        let mut watcher = RecommendedWatcher::new(
            move |result: notify::Result<Event>| {
                if let Ok(event) = result {
                    for event in SyncWatchEvent::from_notify_event(event) {
                        on_event(event);
                    }
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(DEFAULT_WATCH_DEBOUNCE_MS)),
        )
        .map_err(|source| SyncError::Watch {
            operation: "create watcher",
            source,
        })?;

        for root in &roots {
            watcher
                .watch(&root.local_path, RecursiveMode::Recursive)
                .map_err(|source| SyncError::Watch {
                    operation: "watch sync root",
                    source,
                })?;
        }

        Ok(Self {
            _watcher: watcher,
            roots,
        })
    }

    pub fn roots(&self) -> &[SyncRoot] {
        &self.roots
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyncWatchEvent {
    pub kind: SyncWatchEventKind,
    pub path: PathBuf,
}

impl SyncWatchEvent {
    pub fn created(path: impl Into<PathBuf>) -> Self {
        Self {
            kind: SyncWatchEventKind::Created,
            path: path.into(),
        }
    }

    pub fn modified(path: impl Into<PathBuf>) -> Self {
        Self {
            kind: SyncWatchEventKind::Modified,
            path: path.into(),
        }
    }

    pub fn deleted(path: impl Into<PathBuf>) -> Self {
        Self {
            kind: SyncWatchEventKind::Deleted,
            path: path.into(),
        }
    }

    pub fn renamed(from: impl Into<PathBuf>, to: impl Into<PathBuf>) -> Self {
        Self {
            kind: SyncWatchEventKind::Renamed { from: from.into() },
            path: to.into(),
        }
    }

    fn from_notify_event(event: Event) -> Vec<Self> {
        match event.kind {
            EventKind::Create(_) => event.paths.into_iter().map(Self::created).collect(),
            EventKind::Modify(ModifyKind::Name(RenameMode::Both)) if event.paths.len() >= 2 => {
                vec![Self::renamed(
                    event.paths[0].clone(),
                    event.paths[1].clone(),
                )]
            }
            EventKind::Modify(_) => event.paths.into_iter().map(Self::modified).collect(),
            EventKind::Remove(_) => event.paths.into_iter().map(Self::deleted).collect(),
            _ => Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyncWatchEventKind {
    Created,
    Modified,
    Deleted,
    Renamed { from: PathBuf },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyncWatchPlan {
    pub changed_text_paths: Vec<String>,
    pub changed_binary_paths: Vec<String>,
    pub deleted_paths: Vec<String>,
    pub renamed_paths: Vec<SyncRenamePlan>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyncRenamePlan {
    pub from_relative_path: String,
    pub to_relative_path: String,
}

impl SyncWatchPlan {
    pub fn is_empty(&self) -> bool {
        self.changed_text_paths.is_empty()
            && self.changed_binary_paths.is_empty()
            && self.deleted_paths.is_empty()
            && self.renamed_paths.is_empty()
    }
}

pub struct SyncWatchPlanner {
    device: SyncDeviceInfo,
    debounce_window: Duration,
    known_hashes: BTreeMap<String, String>,
    pending: Vec<SyncWatchEvent>,
}

impl SyncWatchPlanner {
    pub fn new(device: SyncDeviceInfo) -> Self {
        Self {
            device,
            debounce_window: Duration::from_millis(DEFAULT_WATCH_DEBOUNCE_MS),
            known_hashes: BTreeMap::new(),
            pending: Vec::new(),
        }
    }

    pub fn with_debounce_window(mut self, debounce_window: Duration) -> Self {
        self.debounce_window = debounce_window;
        self
    }

    pub fn remember_record(&mut self, record: &SyncDocumentRecord) {
        self.known_hashes
            .insert(record.relative_path.clone(), record.content_hash.clone());
    }

    pub fn remember_object_manifest(&mut self, manifest: &SyncObjectManifest) {
        self.known_hashes.insert(
            manifest.relative_path.clone(),
            manifest.content_hash.clone(),
        );
    }

    pub fn known_content_hash(&self, relative_path: &str) -> Option<&str> {
        self.known_hashes
            .get(relative_path)
            .map(std::string::String::as_str)
    }

    pub fn forget_path(&mut self, relative_path: &str) {
        self.known_hashes.remove(relative_path);
    }

    pub fn push(&mut self, event: SyncWatchEvent) {
        self.pending.push(event);
    }

    pub fn debounce_window(&self) -> Duration {
        self.debounce_window
    }

    pub fn drain_plan(&mut self) -> SyncResult<SyncWatchPlan> {
        let mut changed_text = BTreeSet::new();
        let mut changed_binary = BTreeSet::new();
        let mut deleted = BTreeSet::new();
        let mut renamed = Vec::new();

        for event in self.pending.drain(..) {
            match event.kind {
                SyncWatchEventKind::Created | SyncWatchEventKind::Modified => {
                    let relative_path = self.device.home_relative_path(&event.path)?;
                    let relative_path = normalize_home_relative_path(&relative_path)?;
                    if is_utf8_file(&event.path) {
                        changed_text.insert(relative_path);
                    } else if event.path.is_file() {
                        changed_binary.insert(relative_path);
                    }
                }
                SyncWatchEventKind::Deleted => {
                    let relative_path = self.device.home_relative_path(&event.path)?;
                    deleted.insert(normalize_home_relative_path(&relative_path)?);
                }
                SyncWatchEventKind::Renamed { from } => {
                    let from_relative_path = self.device.home_relative_path(&from)?;
                    let to_relative_path = self.device.home_relative_path(&event.path)?;
                    renamed.push(SyncRenamePlan {
                        from_relative_path: normalize_home_relative_path(&from_relative_path)?,
                        to_relative_path: normalize_home_relative_path(&to_relative_path)?,
                    });
                    let to_relative_path = normalize_home_relative_path(&to_relative_path)?;
                    if is_utf8_file(&event.path) {
                        changed_text.insert(to_relative_path);
                    } else if event.path.is_file() {
                        changed_binary.insert(to_relative_path);
                    }
                    deleted.insert(normalize_home_relative_path(&from_relative_path)?);
                }
            }
        }

        Ok(SyncWatchPlan {
            changed_text_paths: changed_text.into_iter().collect(),
            changed_binary_paths: changed_binary.into_iter().collect(),
            deleted_paths: deleted.into_iter().collect(),
            renamed_paths: renamed,
        })
    }

    pub fn apply_plan(
        &mut self,
        engine: &mut SyncEngine,
        plan: &SyncWatchPlan,
    ) -> SyncResult<Vec<String>> {
        let mut changed_text_paths = Vec::new();
        for relative_path in &plan.changed_text_paths {
            let local_path = self.device.local_path_for_home_relative(relative_path)?;
            if !is_utf8_file(&local_path) {
                continue;
            }
            let text = std::fs::read_to_string(&local_path).map_err(|source| SyncError::Io {
                path: local_path.clone(),
                source,
            })?;
            let record = engine.apply_local_text(&local_path, &text)?;
            if self.known_hashes.get(relative_path) == Some(&record.content_hash) {
                continue;
            }
            self.remember_record(&record);
            changed_text_paths.push(relative_path.clone());
        }
        for relative_path in &plan.deleted_paths {
            self.forget_path(relative_path);
        }
        Ok(changed_text_paths)
    }
}

fn is_utf8_file(path: &Path) -> bool {
    std::fs::read(path)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .is_some()
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf, time::Duration};

    use crate::{
        SyncEngine,
        sync_model::SyncDeviceInfo,
        sync_watcher::{SyncWatchEvent, SyncWatchPlanner},
    };

    #[test]
    fn planner_coalesces_duplicate_modify_events_to_home_relative_path()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let home_dir = temp_dir.path().join("home-a");
        let file_path = home_dir.join("az-sync/a.txt");
        fs::create_dir_all(file_path.parent().expect("test file parent"))?;
        fs::write(&file_path, "hello")?;
        let device = SyncDeviceInfo::new("mac-a", home_dir);
        let mut planner =
            SyncWatchPlanner::new(device).with_debounce_window(Duration::from_millis(10));
        planner.push(SyncWatchEvent::modified(&file_path));
        planner.push(SyncWatchEvent::modified(&file_path));

        let plan = planner.drain_plan()?;

        assert_eq!(plan.changed_text_paths, vec!["az-sync/a.txt"]);
        assert!(plan.changed_binary_paths.is_empty());
        assert_eq!(planner.debounce_window(), Duration::from_millis(10));
        Ok(())
    }

    #[test]
    fn planner_preserves_rename_source_and_destination() -> Result<(), Box<dyn std::error::Error>> {
        let device = SyncDeviceInfo::new("mac-a", PathBuf::from("/tmp/home-a"));
        let mut planner = SyncWatchPlanner::new(device);
        planner.push(SyncWatchEvent::renamed(
            "/tmp/home-a/az-sync/old.txt",
            "/tmp/home-a/az-sync/new.txt",
        ));

        let plan = planner.drain_plan()?;

        assert_eq!(plan.renamed_paths[0].from_relative_path, "az-sync/old.txt");
        assert_eq!(plan.renamed_paths[0].to_relative_path, "az-sync/new.txt");
        Ok(())
    }

    #[test]
    fn planner_applies_changed_utf8_file_to_crdt_engine() -> Result<(), Box<dyn std::error::Error>>
    {
        let temp_dir = tempfile::tempdir()?;
        let home_dir = temp_dir.path().join("home-a");
        let file_path = home_dir.join("az-sync/a.txt");
        fs::create_dir_all(file_path.parent().expect("test file parent"))?;
        fs::write(&file_path, "one\ntwo\nthree")?;
        let device = SyncDeviceInfo::new("mac-a", home_dir.clone());
        let mut engine = SyncEngine::with_device(device.clone());
        let mut planner = SyncWatchPlanner::new(device);
        planner.push(SyncWatchEvent::modified(file_path));
        let plan = planner.drain_plan()?;

        planner.apply_plan(&mut engine, &plan)?;

        assert_eq!(engine.materialize_text("az-sync/a.txt")?, "one\ntwo\nthree");
        Ok(())
    }

    #[test]
    fn planner_classifies_non_utf8_file_as_binary_change() -> Result<(), Box<dyn std::error::Error>>
    {
        let temp_dir = tempfile::tempdir()?;
        let home_dir = temp_dir.path().join("home-a");
        let file_path = home_dir.join("az-sync/blob.bin");
        fs::create_dir_all(file_path.parent().expect("test file parent"))?;
        fs::write(&file_path, [0, 159, 146, 150])?;
        let device = SyncDeviceInfo::new("mac-a", home_dir);
        let mut planner = SyncWatchPlanner::new(device);
        planner.push(SyncWatchEvent::modified(file_path));

        let plan = planner.drain_plan()?;

        assert!(plan.changed_text_paths.is_empty());
        assert_eq!(plan.changed_binary_paths, vec!["az-sync/blob.bin"]);
        Ok(())
    }

    #[test]
    fn planner_remembers_binary_object_manifest_hash() -> Result<(), Box<dyn std::error::Error>> {
        let device = SyncDeviceInfo::new("mac-a", PathBuf::from("/tmp/home-a"));
        let mut planner = SyncWatchPlanner::new(device);
        let manifest =
            crate::SyncObjectManifest::plan("main", "az-sync/blob.bin", "sha256:demo", 4, 2)?;

        planner.remember_object_manifest(&manifest);

        assert_eq!(
            planner.known_content_hash("az-sync/blob.bin"),
            Some("sha256:demo")
        );
        Ok(())
    }
}
