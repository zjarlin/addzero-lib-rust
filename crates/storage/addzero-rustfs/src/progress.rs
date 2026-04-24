use crate::types::ObjectMetadata;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub trait UploadProgressListener: Send + Sync {
    fn on_progress(&self, progress: UploadProgressData);
}

#[derive(Debug, Clone, PartialEq)]
pub struct UploadProgressData {
    pub uploaded: u64,
    pub total: u64,
    pub percent: f64,
    pub part_number: Option<u32>,
    pub total_parts: Option<u32>,
}

#[derive(Clone)]
pub struct MultipartUploadConfig {
    pub part_size: u64,
    pub concurrency: usize,
    pub max_retries: usize,
    pub timeout_seconds: u64,
    pub multipart_threshold: u64,
    pub progress_listener: Option<Arc<dyn UploadProgressListener>>,
}

impl MultipartUploadConfig {
    pub const DEFAULT_PART_SIZE: u64 = 5 * 1024 * 1024;
    pub const DEFAULT_CONCURRENCY: usize = 3;
    pub const DEFAULT_MAX_RETRIES: usize = 3;
    pub const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
    pub const DEFAULT_MULTIPART_THRESHOLD: u64 = 100 * 1024 * 1024;
}

impl Default for MultipartUploadConfig {
    fn default() -> Self {
        Self {
            part_size: Self::DEFAULT_PART_SIZE,
            concurrency: Self::DEFAULT_CONCURRENCY,
            max_retries: Self::DEFAULT_MAX_RETRIES,
            timeout_seconds: Self::DEFAULT_TIMEOUT_SECONDS,
            multipart_threshold: Self::DEFAULT_MULTIPART_THRESHOLD,
            progress_listener: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartInfo {
    pub part_number: u32,
    pub start: u64,
    pub end: u64,
    pub size: u64,
    pub etag: Option<String>,
    pub status: PartStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartStatus {
    Pending,
    Uploading,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UploadStatus {
    pub upload_id: String,
    pub bucket_name: String,
    pub object_key: String,
    pub file_size: u64,
    pub uploaded_size: u64,
    pub progress: f64,
    pub parts: Vec<PartInfo>,
    pub status: UploadStatusType,
    pub error: Option<String>,
    pub created_at_millis: u128,
    pub updated_at_millis: u128,
}

impl UploadStatus {
    pub fn calculate_progress(&self) -> f64 {
        if self.file_size == 0 {
            return 0.0;
        }
        ((self.uploaded_size as f64 / self.file_size as f64) * 100.0).min(100.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadStatusType {
    Initialized,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MultipartUploadResult {
    Success {
        bucket_name: String,
        object_key: String,
        upload_id: String,
        etag: String,
        file_size: u64,
        parts_count: usize,
    },
    Failed {
        bucket_name: String,
        object_key: String,
        upload_id: Option<String>,
        error: String,
    },
    InProgress {
        upload_id: String,
        status: UploadStatus,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct UploadProgress {
    pub total_bytes: u64,
    pub uploaded_bytes: u64,
    pub percent: f64,
    pub current_part: Option<u32>,
    pub total_parts: Option<u32>,
    pub speed: Option<u64>,
    pub remaining_seconds: Option<u64>,
}

impl UploadProgress {
    pub fn is_complete(&self) -> bool {
        self.percent >= 100.0
    }

    pub fn formatted(&self) -> String {
        let mut formatted = format!(
            "{:.2}% ({}/{})",
            self.percent,
            Self::format_bytes(self.uploaded_bytes),
            Self::format_bytes(self.total_bytes)
        );
        if let (Some(current), Some(total)) = (self.current_part, self.total_parts) {
            formatted.push_str(&format!(" ({current}/{total})"));
        }
        if let Some(speed) = self.speed {
            formatted.push_str(&format!(" @ {}/s", Self::format_bytes(speed)));
        }
        if let Some(remaining) = self.remaining_seconds {
            formatted.push_str(&format!(" {} remaining", Self::format_seconds(remaining)));
        }
        formatted
    }

    pub fn format_bytes(bytes: u64) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;
        let bytes_f = bytes as f64;
        if bytes_f < KB {
            format!("{bytes} B")
        } else if bytes_f < MB {
            format!("{:.2} KB", bytes_f / KB)
        } else if bytes_f < GB {
            format!("{:.2} MB", bytes_f / MB)
        } else {
            format!("{:.2} GB", bytes_f / GB)
        }
    }

    pub fn format_seconds(seconds: u64) -> String {
        if seconds < 60 {
            format!("{seconds}s")
        } else if seconds < 3600 {
            format!("{}m {}s", seconds / 60, seconds % 60)
        } else {
            format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
        }
    }
}

pub trait UploadProgressStorage: Send + Sync {
    fn save_status(&self, key: &str, status: UploadStatus) -> bool;
    fn get_status(&self, key: &str) -> Option<UploadStatus>;
    fn delete_status(&self, key: &str) -> bool;
    fn update_part_status(
        &self,
        key: &str,
        part_number: u32,
        status: PartStatus,
        etag: Option<String>,
    ) -> bool;
    fn update_uploaded_size(&self, key: &str, uploaded_size: u64) -> bool;
}

#[derive(Debug, Default)]
pub struct InMemoryUploadProgressStorage {
    state: Mutex<HashMap<String, UploadStatus>>,
}

impl InMemoryUploadProgressStorage {
    pub fn generate_key(bucket_name: &str, object_key: &str) -> String {
        format!("upload:progress:{bucket_name}:{object_key}")
    }

    pub fn generate_upload_id_key(upload_id: &str) -> String {
        format!("upload:progress:id:{upload_id}")
    }
}

impl UploadProgressStorage for InMemoryUploadProgressStorage {
    fn save_status(&self, key: &str, status: UploadStatus) -> bool {
        self.state
            .lock()
            .expect("upload progress storage mutex should not be poisoned")
            .insert(key.to_owned(), status);
        true
    }

    fn get_status(&self, key: &str) -> Option<UploadStatus> {
        self.state
            .lock()
            .expect("upload progress storage mutex should not be poisoned")
            .get(key)
            .cloned()
    }

    fn delete_status(&self, key: &str) -> bool {
        self.state
            .lock()
            .expect("upload progress storage mutex should not be poisoned")
            .remove(key)
            .is_some()
    }

    fn update_part_status(
        &self,
        key: &str,
        part_number: u32,
        status: PartStatus,
        etag: Option<String>,
    ) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("upload progress storage mutex should not be poisoned");
        let Some(current) = state.get_mut(key) else {
            return false;
        };

        current.parts = current
            .parts
            .iter()
            .cloned()
            .map(|part| {
                if part.part_number == part_number {
                    PartInfo {
                        status,
                        etag: etag.clone().or(part.etag),
                        ..part
                    }
                } else {
                    part
                }
            })
            .collect();
        current.uploaded_size = current
            .parts
            .iter()
            .filter(|part| part.status == PartStatus::Completed)
            .map(|part| part.size)
            .sum();
        current.progress = current.calculate_progress();
        current.updated_at_millis = now_millis();
        true
    }

    fn update_uploaded_size(&self, key: &str, uploaded_size: u64) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("upload progress storage mutex should not be poisoned");
        let Some(current) = state.get_mut(key) else {
            return false;
        };
        current.uploaded_size = uploaded_size;
        current.progress = current.calculate_progress();
        current.updated_at_millis = now_millis();
        true
    }
}

pub struct SpeedTrackingProgressListener {
    progress_storage: Option<Arc<dyn UploadProgressStorage>>,
    bucket_name: Option<String>,
    object_key: Option<String>,
    on_update: Box<dyn Fn(UploadProgress) + Send + Sync>,
    last_sample: Mutex<Option<(Instant, u64)>>,
}

impl SpeedTrackingProgressListener {
    pub fn new(
        progress_storage: Option<Arc<dyn UploadProgressStorage>>,
        bucket_name: Option<String>,
        object_key: Option<String>,
        on_update: impl Fn(UploadProgress) + Send + Sync + 'static,
    ) -> Self {
        Self {
            progress_storage,
            bucket_name,
            object_key,
            on_update: Box::new(on_update),
            last_sample: Mutex::new(None),
        }
    }

    pub fn reset(&self) {
        *self
            .last_sample
            .lock()
            .expect("progress listener mutex should not be poisoned") = None;
    }
}

impl UploadProgressListener for SpeedTrackingProgressListener {
    fn on_progress(&self, progress: UploadProgressData) {
        let mut last_sample = self
            .last_sample
            .lock()
            .expect("progress listener mutex should not be poisoned");
        let now = Instant::now();

        let speed = last_sample.as_ref().and_then(|(instant, uploaded)| {
            let elapsed = now.duration_since(*instant).as_secs_f64();
            if elapsed <= 0.0 || progress.uploaded < *uploaded {
                None
            } else {
                Some(((progress.uploaded - *uploaded) as f64 / elapsed) as u64)
            }
        });
        *last_sample = Some((now, progress.uploaded));

        if let (Some(storage), Some(bucket), Some(key)) = (
            self.progress_storage.as_ref(),
            self.bucket_name.as_deref(),
            self.object_key.as_deref(),
        ) {
            let storage_key = InMemoryUploadProgressStorage::generate_key(bucket, key);
            let _ = storage.update_uploaded_size(&storage_key, progress.uploaded);
        }

        let remaining_seconds = speed.map(|speed| {
            if speed == 0 || progress.total <= progress.uploaded {
                0
            } else {
                (progress.total - progress.uploaded) / speed
            }
        });

        (self.on_update)(UploadProgress {
            total_bytes: progress.total,
            uploaded_bytes: progress.uploaded,
            percent: progress.percent,
            current_part: progress.part_number,
            total_parts: progress.total_parts,
            speed,
            remaining_seconds,
        });
    }
}

pub fn now_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_millis()
}

#[allow(dead_code)]
pub fn metadata_from_objects(objects: &[ObjectMetadata]) -> Vec<String> {
    objects.iter().map(|object| object.key.clone()).collect()
}
