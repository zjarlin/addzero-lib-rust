use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, anyhow, bail};
use az_str::sanitize::{to_slash_path, to_slug};

use crate::{
    backend::{
        dotfiles_monitor_diff::{changed_ranges, first_overlap, snippet},
        dotfiles_monitor_types::{
            DotfilesBaselineEntry, DotfilesConflict, DotfilesDevicesRequest, DotfilesMonitorStatus,
            DotfilesPeerDevice, DotfilesPeerDeviceInput, DotfilesWatchedFile,
            ResolveDotfilesConflictRequest,
        },
        paths::config_center_state_dir_path,
    },
};

const DEVICES_FILE_NAME: &str = "dotfiles-devices.json";
const MAX_FILE_BYTES: u64 = 256 * 1024;
const SKIP_DIRS: &[&str] = &[
    ".git",
    "__pycache__",
    "node_modules",
    "target",
    ".bootstrap-backups",
];
const SKIP_FILE_NAMES: &[&str] = &[".DS_Store"];
const SKIP_NAME_SUFFIXES: &[&str] = &[".mdb"];
const SKIP_FILE_EXTENSIONS: &[&str] = &["db", "mdb", "sqlite", "sqlite3"];

pub fn scan_dotfiles_status() -> anyhow::Result<DotfilesMonitorStatus> {
    let context = ScanContext::load()?;
    let baseline = load_baseline(&context.baseline_path)?;
    let mut next_baseline = baseline.clone();
    let mut pending_files = Vec::new();
    let mut conflicts = Vec::new();
    let repo_files = collect_files(&context.source_home)?;

    for repo_path in &repo_files {
        let relative = repo_path.strip_prefix(&context.source_home)?.to_path_buf();
        let relative_key = to_slash_path(&relative);
        let repo_content = read_small_text(repo_path)?;
        let repo_modified = modified_secs(repo_path).unwrap_or_default();
        let base = baseline.get(&relative_key);
        let repo_changed = base.is_some_and(|entry| entry.content != repo_content);
        let target_states = collect_target_states(&context.targets, &relative, base);

        for state in &target_states {
            let file = classify_target(
                &relative_key,
                repo_path,
                &repo_content,
                repo_modified,
                state,
                base,
            );
            if file.status != "same" {
                pending_files.push(file);
            }
        }

        if !repo_changed {
            conflicts.extend(classify_device_conflicts(
                &relative_key,
                repo_path,
                base,
                &target_states,
            ));
        }

        for state in &target_states {
            if let Some(conflict) = classify_repo_conflict(
                &relative_key,
                repo_path,
                &repo_content,
                repo_modified,
                state,
                base,
            ) {
                conflicts.push(conflict);
            }
        }

        if should_refresh_baseline(base, &repo_content, &target_states) {
            upsert_baseline(
                &mut next_baseline,
                relative_key,
                repo_content,
                repo_modified,
                latest_target_modified(&target_states),
            );
        }
    }

    save_baseline(&context.baseline_path, &next_baseline)?;
    Ok(DotfilesMonitorStatus {
        root: context.root.to_string_lossy().into_owned(),
        source_home: context.source_home.to_string_lossy().into_owned(),
        home: context.local_home.to_string_lossy().into_owned(),
        baseline_path: context.baseline_path.to_string_lossy().into_owned(),
        devices: context.devices,
        watched_files: repo_files.len(),
        changed_files: pending_files.len(),
        conflict_files: conflicts.len(),
        pending_files,
        conflicts,
        updated_at: now_text(),
    })
}

pub fn save_dotfiles_devices(
    request: DotfilesDevicesRequest,
) -> anyhow::Result<DotfilesMonitorStatus> {
    let existing = load_devices()?;
    let devices = request
        .devices
        .into_iter()
        .filter_map(|device| normalize_input_device(device, &existing))
        .collect::<Vec<_>>();
    write_devices(&devices)?;
    scan_dotfiles_status()
}

pub fn resolve_dotfiles_conflict(
    request: ResolveDotfilesConflictRequest,
) -> anyhow::Result<DotfilesMonitorStatus> {
    let status = scan_dotfiles_status()?;
    let conflict = status
        .conflicts
        .into_iter()
        .find(|item| item.id == request.conflict_id)
        .ok_or_else(|| anyhow!("conflict not found: {}", request.conflict_id))?;
    let resolved_text = match request.strategy.as_str() {
        "left" => read_small_text(Path::new(&conflict.left_path))?,
        "right" => read_small_text(Path::new(&conflict.right_path))?,
        "manual" => {
            let merged = request
                .merged_text
                .filter(|text| !text.trim().is_empty())
                .ok_or_else(|| anyhow!("manual merge content is empty"))?;
            let left_content = read_small_text(Path::new(&conflict.left_path))?;
            replace_line_range(
                &left_content,
                conflict.line_start,
                conflict.line_end,
                &merged,
            )
        }
        other => {
            let message = format!("unsupported conflict resolution strategy: {other}");
            bail!(message);
        }
    };

    let write_paths = [conflict.repo_path, conflict.left_path, conflict.right_path]
        .into_iter()
        .chain(peer_paths_for_relative(&conflict.relative_path)?)
        .collect::<BTreeSet<_>>();
    for path in &write_paths {
        write_text_file(Path::new(path), &resolved_text)?;
    }

    let baseline_path = baseline_path()?;
    let mut baseline = load_baseline(&baseline_path)?;
    upsert_baseline(
        &mut baseline,
        conflict.relative_path,
        resolved_text,
        current_epoch_secs(),
        current_epoch_secs(),
    );
    save_baseline(&baseline_path, &baseline)?;
    scan_dotfiles_status()
}

struct ScanContext {
    root: PathBuf,
    source_home: PathBuf,
    local_home: PathBuf,
    baseline_path: PathBuf,
    devices: Vec<DotfilesPeerDevice>,
    targets: Vec<TargetRoot>,
}

#[derive(Clone)]
struct TargetRoot {
    id: String,
    name: String,
    home_root: PathBuf,
}

struct TargetState {
    target: TargetRoot,
    target_path: PathBuf,
    content: String,
    modified: u64,
}

impl ScanContext {
    fn load() -> anyhow::Result<Self> {
        let root = dotfiles_root()?;
        let source_home = root.join("home");
        let local_home = home_dir()?;
        let baseline_path = baseline_path()?;
        let devices = refresh_devices(load_devices()?);
        let mut targets = vec![TargetRoot {
            id: "local-home".to_string(),
            name: "current-node".to_string(),
            home_root: local_home.clone(),
        }];

        for device in &devices {
            if !device.enabled {
                continue;
            }
            let device_root = PathBuf::from(&device.home_path);
            if device_root.is_absolute() && device_root.exists() {
                targets.push(TargetRoot {
                    id: device.id.clone(),
                    name: device.name.clone(),
                    home_root: device_root,
                });
            }
        }

        Ok(Self {
            root,
            source_home,
            local_home,
            baseline_path,
            devices,
            targets,
        })
    }
}

fn collect_target_states(
    targets: &[TargetRoot],
    relative: &Path,
    base: Option<&DotfilesBaselineEntry>,
) -> Vec<TargetState> {
    targets
        .iter()
        .map(|target| {
            let target_path = target.home_root.join(relative);
            let content = read_small_text(&target_path).unwrap_or_default();
            let modified = modified_secs(&target_path).unwrap_or_default();
            let _changed = base.is_some_and(|entry| entry.content != content);
            TargetState {
                target: target.clone(),
                target_path,
                content,
                modified,
            }
        })
        .collect()
}

fn classify_target(
    relative: &str,
    repo_path: &Path,
    repo_content: &str,
    _repo_modified: u64,
    state: &TargetState,
    base: Option<&DotfilesBaselineEntry>,
) -> DotfilesWatchedFile {
    let (status, detail) = match base {
        None => (
            "baseline",
            "first observation; baseline will be established.".to_string(),
        ),
        Some(_) if repo_content == state.content => (
            "same",
            "dotfiles and target content are aligned.".to_string(),
        ),
        Some(entry) => classify_target_detail(repo_content, &state.content, entry),
    };

    DotfilesWatchedFile {
        relative_path: relative.to_string(),
        repo_path: repo_path.to_string_lossy().into_owned(),
        target_path: state.target_path.to_string_lossy().into_owned(),
        target_name: state.target.name.clone(),
        status: status.to_string(),
        detail,
    }
}

fn classify_target_detail(
    repo_content: &str,
    target_content: &str,
    base: &DotfilesBaselineEntry,
) -> (&'static str, String) {
    let repo_changed = repo_content != base.content;
    let target_changed = target_content != base.content;
    if !repo_changed || !target_changed {
        return (
            "one-sided",
            "only one side diverged from baseline; sync can handle it.".to_string(),
        );
    }

    let repo_ranges = changed_ranges(&base.content, repo_content);
    let target_ranges = changed_ranges(&base.content, target_content);
    if let Some(overlap) = first_overlap(&repo_ranges, &target_ranges) {
        return (
            "line-conflict",
            format!(
                "both sides changed baseline lines {}-{}.",
                overlap.start, overlap.end
            ),
        );
    }

    (
        "mergeable",
        "changes do not overlap and can be merged.".to_string(),
    )
}

fn classify_repo_conflict(
    relative: &str,
    repo_path: &Path,
    repo_content: &str,
    repo_modified: u64,
    state: &TargetState,
    base: Option<&DotfilesBaselineEntry>,
) -> Option<DotfilesConflict> {
    let entry = base?;
    let repo_changed = repo_content != entry.content;
    let target_changed = state.content != entry.content;
    if !repo_changed || !target_changed {
        return None;
    }

    let repo_ranges = changed_ranges(&entry.content, repo_content);
    let target_ranges = changed_ranges(&entry.content, &state.content);
    let overlap = first_overlap(&repo_ranges, &target_ranges)?;
    Some(build_conflict(
        format!("dotfiles-{}-{}", to_slug(relative), to_slug(&state.target.id)),
        relative,
        repo_path,
        &state.target_path,
        &state.target_path,
        &state.target.name,
        "shared-baseline-copy",
        &state.content,
        repo_content,
        state.modified,
        repo_modified,
        &entry.content,
        overlap.start,
        overlap.end,
        format!(
            "{} and the shared baseline copy changed the same relative path with overlapping lines.",
            state.target.name
        ),
    ))
}

fn classify_device_conflicts(
    relative: &str,
    repo_path: &Path,
    base: Option<&DotfilesBaselineEntry>,
    target_states: &[TargetState],
) -> Vec<DotfilesConflict> {
    let Some(entry) = base else {
        return Vec::new();
    };
    let changed_states = target_states
        .iter()
        .filter(|state| state.content != entry.content)
        .collect::<Vec<_>>();
    let mut conflicts = Vec::new();

    for left_index in 0..changed_states.len() {
        for right_index in (left_index + 1)..changed_states.len() {
            let left = changed_states[left_index];
            let right = changed_states[right_index];
            if left.content == right.content {
                continue;
            }
            let left_ranges = changed_ranges(&entry.content, &left.content);
            let right_ranges = changed_ranges(&entry.content, &right.content);
            let Some(overlap) = first_overlap(&left_ranges, &right_ranges) else {
                continue;
            };
            conflicts.push(build_conflict(
                format!(
                    "dotfiles-{}-{}-{}",
                    to_slug(relative),
                    to_slug(&left.target.id),
                    to_slug(&right.target.id)
                ),
                relative,
                repo_path,
                &left.target_path,
                &right.target_path,
                &left.target.name,
                &right.target.name,
                &left.content,
                &right.content,
                left.modified,
                right.modified,
                &entry.content,
                overlap.start,
                overlap.end,
                format!(
                    "{} and {} changed the same relative path with overlapping lines.",
                    left.target.name, right.target.name
                ),
            ));
        }
    }

    conflicts
}

#[allow(clippy::too_many_arguments)]
fn build_conflict(
    id: String,
    relative: &str,
    repo_path: &Path,
    left_path: &Path,
    right_path: &Path,
    left_label: &str,
    right_label: &str,
    left_content: &str,
    right_content: &str,
    left_modified: u64,
    right_modified: u64,
    base_content: &str,
    line_start: usize,
    line_end: usize,
    reason: String,
) -> DotfilesConflict {
    let actual_left_path = if left_label == "shared-baseline-copy" {
        repo_path.to_string_lossy().into_owned()
    } else {
        left_path.to_string_lossy().into_owned()
    };
    let actual_right_path = if right_label == "shared-baseline-copy" {
        repo_path.to_string_lossy().into_owned()
    } else {
        right_path.to_string_lossy().into_owned()
    };

    DotfilesConflict {
        id,
        relative_path: relative.to_string(),
        repo_path: repo_path.to_string_lossy().into_owned(),
        left_label: left_label.to_string(),
        right_label: right_label.to_string(),
        left_path: actual_left_path,
        right_path: actual_right_path,
        title: format!("~/{relative}"),
        reason,
        risk: "line-level conflict".to_string(),
        risk_class: "risk".to_string(),
        suggestion: "Choose the left side, right side, or merge manually.".to_string(),
        local_time: unix_text(left_modified),
        remote_time: unix_text(right_modified),
        local_text: snippet(left_content, line_start, line_end),
        remote_text: snippet(right_content, line_start, line_end),
        base_text: snippet(base_content, line_start, line_end),
        line_start,
        line_end,
    }
}

fn should_refresh_baseline(
    base: Option<&DotfilesBaselineEntry>,
    repo_content: &str,
    target_states: &[TargetState],
) -> bool {
    if base.is_none() {
        return true;
    }
    target_states
        .iter()
        .all(|state| state.content == repo_content)
}

fn latest_target_modified(target_states: &[TargetState]) -> u64 {
    target_states
        .iter()
        .map(|state| state.modified)
        .max()
        .unwrap_or_default()
}

fn load_devices() -> anyhow::Result<Vec<DotfilesPeerDevice>> {
    let path = devices_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&path)
        .with_context(|| format!("read dotfiles devices: {}", path.display()))?;
    serde_json::from_str(&text)
        .with_context(|| format!("parse dotfiles devices: {}", path.display()))
}

fn refresh_devices(devices: Vec<DotfilesPeerDevice>) -> Vec<DotfilesPeerDevice> {
    devices
        .into_iter()
        .map(|device| DotfilesPeerDevice {
            last_seen: if device.enabled && Path::new(&device.home_path).exists() {
                now_text()
            } else {
                device.last_seen
            },
            ..device
        })
        .collect()
}

fn write_devices(devices: &[DotfilesPeerDevice]) -> anyhow::Result<()> {
    let path = devices_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create dotfiles devices dir: {}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(devices)?;
    fs::write(&path, text).with_context(|| format!("write dotfiles devices: {}", path.display()))
}

fn normalize_input_device(
    device: DotfilesPeerDeviceInput,
    existing: &[DotfilesPeerDevice],
) -> Option<DotfilesPeerDevice> {
    let id = device.id.trim().to_string();
    let home_path = device.home_path.trim().to_string();
    if id.is_empty() || home_path.is_empty() {
        return None;
    }
    let previous = existing.iter().find(|item| item.id == id);
    let name = if device.name.trim().is_empty() {
        home_path.clone()
    } else {
        device.name.trim().to_string()
    };
    Some(DotfilesPeerDevice {
        id,
        name,
        home_path,
        enabled: device.enabled,
        last_seen: previous
            .map(|item| item.last_seen.clone())
            .unwrap_or_else(now_text),
    })
}

fn peer_paths_for_relative(relative: &str) -> anyhow::Result<Vec<String>> {
    Ok(load_devices()?
        .into_iter()
        .filter(|device| device.enabled)
        .map(|device| {
            PathBuf::from(device.home_path)
                .join(relative)
                .to_string_lossy()
                .into_owned()
        })
        .collect())
}

fn devices_path() -> anyhow::Result<PathBuf> {
    Ok(config_center_state_dir_path()?.join(DEVICES_FILE_NAME))
}

fn baseline_path() -> anyhow::Result<PathBuf> {
    Ok(config_center_state_dir_path()?.join("dotfiles-baseline.json"))
}

fn dotfiles_root() -> anyhow::Result<PathBuf> {
    Ok(home_dir()?.join("aio").join("Dotfiles"))
}

fn collect_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files_inner(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files_inner(path: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    if should_skip(path) || !path.exists() {
        return Ok(());
    }
    for entry in
        fs::read_dir(path).with_context(|| format!("read dotfiles dir: {}", path.display()))?
    {
        let path = entry?.path();
        if path.is_dir() {
            collect_files_inner(&path, files)?;
        } else if path.is_file() && is_small_text_candidate(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn load_baseline(path: &Path) -> anyhow::Result<BTreeMap<String, DotfilesBaselineEntry>> {
    if !path.exists() {
        return Ok(BTreeMap::new());
    }
    let text = fs::read_to_string(path)
        .with_context(|| format!("read dotfiles baseline: {}", path.display()))?;
    serde_json::from_str(&text)
        .with_context(|| format!("parse dotfiles baseline: {}", path.display()))
}

fn save_baseline(
    path: &Path,
    baseline: &BTreeMap<String, DotfilesBaselineEntry>,
) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create dotfiles baseline dir: {}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(baseline)?;
    fs::write(path, text).with_context(|| format!("write dotfiles baseline: {}", path.display()))
}

fn upsert_baseline(
    baseline: &mut BTreeMap<String, DotfilesBaselineEntry>,
    relative_path: String,
    content: String,
    repo_modified: u64,
    home_modified: u64,
) {
    baseline.insert(
        relative_path.clone(),
        DotfilesBaselineEntry {
            relative_path,
            content,
            repo_modified,
            home_modified,
        },
    );
}

fn write_text_file(path: &Path, content: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create parent dir: {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("write file: {}", path.display()))
}

fn replace_line_range(content: &str, start: usize, end: usize, replacement: &str) -> String {
    let original_lines = content.lines().collect::<Vec<_>>();
    let start_index = start.saturating_sub(1).min(original_lines.len());
    let end_index = end.min(original_lines.len());
    let mut lines = Vec::new();

    lines.extend(
        original_lines[..start_index]
            .iter()
            .map(|line| (*line).to_string()),
    );
    lines.extend(
        replacement
            .trim_end_matches('\n')
            .lines()
            .map(ToString::to_string),
    );
    lines.extend(
        original_lines[end_index..]
            .iter()
            .map(|line| (*line).to_string()),
    );

    let mut text = lines.join("\n");
    text.push('\n');
    text
}

fn read_small_text(path: &Path) -> anyhow::Result<String> {
    if !path.exists() {
        return Ok(String::new());
    }
    let metadata =
        fs::metadata(path).with_context(|| format!("read file metadata: {}", path.display()))?;
    if metadata.len() > MAX_FILE_BYTES {
        let message = format!("file exceeds watch size limit: {}", path.display());
        bail!(message);
    }
    fs::read_to_string(path).with_context(|| format!("read text file: {}", path.display()))
}

fn is_small_text_candidate(path: &Path) -> bool {
    if should_skip_file(path) {
        return false;
    }
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if metadata.len() > MAX_FILE_BYTES {
        return false;
    }
    fs::read(path)
        .map(|bytes| std::str::from_utf8(&bytes).is_ok())
        .unwrap_or(false)
}

fn should_skip(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            SKIP_DIRS.contains(&name)
                || SKIP_NAME_SUFFIXES
                    .iter()
                    .any(|suffix| name.ends_with(suffix))
        })
}

fn should_skip_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            SKIP_FILE_NAMES.contains(&name)
                || path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| {
                        SKIP_FILE_EXTENSIONS
                            .iter()
                            .any(|candidate| extension.eq_ignore_ascii_case(candidate))
                    })
        })
}

fn home_dir() -> anyhow::Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("cannot resolve user home"))
}

fn modified_secs(path: &Path) -> anyhow::Result<u64> {
    let modified = fs::metadata(path)?
        .modified()
        .unwrap_or(SystemTime::UNIX_EPOCH);
    Ok(modified
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs())
}

fn now_text() -> String {
    unix_text(current_epoch_secs())
}

fn unix_text(seconds: u64) -> String {
    format!("unix:{seconds}")
}

fn current_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::collect_files;

    #[test]
    fn collect_files_skips_binary_runtime_artifacts() {
        let root = env::temp_dir().join(format!(
            "config-center-dotfiles-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        fs::create_dir_all(root.join(".config/zed/prompts/prompts-library-db.0.mdb")).unwrap();
        fs::write(root.join(".config/profile.toml"), "theme = 'light'\n").unwrap();
        fs::write(root.join(".DS_Store"), [0x00, 0x9f, 0x01]).unwrap();
        fs::write(root.join("cache.db"), [0xde, 0xad, 0xbe, 0xef]).unwrap();
        fs::write(
            root.join(".config/zed/prompts/prompts-library-db.0.mdb/data.mdb"),
            [0xde, 0xad, 0xbe, 0xef],
        )
        .unwrap();

        let files = collect_files(&root).unwrap();

        assert_eq!(files, vec![root.join(".config/profile.toml")]);
        fs::remove_dir_all(root).unwrap();
    }
}
