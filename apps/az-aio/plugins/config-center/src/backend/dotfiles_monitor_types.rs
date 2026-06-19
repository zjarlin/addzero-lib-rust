
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DotfilesWatchedFile {
    pub relative_path: String,
    pub repo_path: String,
    pub target_path: String,
    pub target_name: String,
    pub status: String,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DotfilesPeerDevice {
    pub id: String,
    pub name: String,
    pub home_path: String,
    pub enabled: bool,
    pub last_seen: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DotfilesBaselineEntry {
    pub relative_path: String,
    pub content: String,
    pub repo_modified: u64,
    pub home_modified: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DotfilesPeerDeviceInput {
    pub id: String,
    pub name: String,
    pub home_path: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DotfilesDevicesRequest {
    pub devices: Vec<DotfilesPeerDeviceInput>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveDotfilesConflictRequest {
    pub conflict_id: String,
    pub strategy: String,
    pub merged_text: Option<String>,
}
