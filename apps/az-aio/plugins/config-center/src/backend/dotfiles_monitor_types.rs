use az_derive_aliases::{
    apply, deserialize_camel_eq, serde_camel_eq, serde_eq, serialize_camel_eq,
};

#[apply(serialize_camel_eq)]
pub struct DotfilesMonitorStatus {
    pub root: String,
    pub source_home: String,
    pub home: String,
    pub baseline_path: String,
    pub devices: Vec<DotfilesPeerDevice>,
    pub watched_files: usize,
    pub changed_files: usize,
    pub conflict_files: usize,
    pub pending_files: Vec<DotfilesWatchedFile>,
    pub conflicts: Vec<DotfilesConflict>,
    pub updated_at: String,
}

#[apply(serialize_camel_eq)]
pub struct DotfilesWatchedFile {
    pub relative_path: String,
    pub repo_path: String,
    pub target_path: String,
    pub target_name: String,
    pub status: String,
    pub detail: String,
}

#[apply(serialize_camel_eq)]
pub struct DotfilesConflict {
    pub id: String,
    pub relative_path: String,
    pub repo_path: String,
    pub left_label: String,
    pub right_label: String,
    pub left_path: String,
    pub right_path: String,
    pub title: String,
    pub reason: String,
    pub risk: String,
    pub risk_class: String,
    pub suggestion: String,
    pub local_time: String,
    pub remote_time: String,
    pub local_text: String,
    pub remote_text: String,
    pub base_text: String,
    pub line_start: usize,
    pub line_end: usize,
}

#[apply(serde_camel_eq)]
pub struct DotfilesPeerDevice {
    pub id: String,
    pub name: String,
    pub home_path: String,
    pub enabled: bool,
    pub last_seen: String,
}

#[apply(serde_eq)]
pub struct DotfilesBaselineEntry {
    pub relative_path: String,
    pub content: String,
    pub repo_modified: u64,
    pub home_modified: u64,
}

#[apply(deserialize_camel_eq)]
pub struct DotfilesPeerDeviceInput {
    pub id: String,
    pub name: String,
    pub home_path: String,
    pub enabled: bool,
}

#[apply(deserialize_camel_eq)]
pub struct DotfilesDevicesRequest {
    pub devices: Vec<DotfilesPeerDeviceInput>,
}

#[apply(deserialize_camel_eq)]
pub struct ResolveDotfilesConflictRequest {
    pub conflict_id: String,
    pub strategy: String,
    pub merged_text: Option<String>,
}
