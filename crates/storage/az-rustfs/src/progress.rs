use crate::types::ObjectMetadata;
use az_derive_aliases::{
    apply, impl_default, plain_clone, plain_code_display_no_default_enum, plain_default_debug,
    plain_eq, plain_partial_eq,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// 从中毒 mutex 中恢复，避免进度状态因回调 panic 后不可读。
fn recover_lock<T>(mutex: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| {
        eprintln!("WARN: upload progress mutex was poisoned, recovering");
        poisoned.into_inner()
    })
}

/// 上传进度监听器。
///
/// 实现方可以把进度写入 UI、日志或外部状态存储。
pub trait UploadProgressListener: Send + Sync {
    /// 接收一次上传进度快照。
    fn on_progress(&self, progress: UploadProgressData);
}

/// 上传进度原始快照。
#[apply(plain_partial_eq)]
pub struct UploadProgressData {
    /// 已上传字节数。
    pub uploaded: u64,
    /// 总字节数。
    pub total: u64,
    /// 完成百分比，范围通常为 `0.0..=100.0`。
    pub percent: f64,
    /// 当前分片编号。
    pub part_number: Option<u32>,
    /// 总分片数。
    pub total_parts: Option<u32>,
}

/// 分片上传配置。
#[apply(plain_clone)]
pub struct MultipartUploadConfig {
    /// 单个分片大小，单位字节。
    pub part_size: u64,
    /// 并发上传分片数。
    pub concurrency: usize,
    /// 单个分片最大重试次数。
    pub max_retries: usize,
    /// 单次上传超时时间，单位秒。
    pub timeout_seconds: u64,
    /// 文件大小达到该阈值后使用分片上传。
    pub multipart_threshold: u64,
    /// 可选进度监听器。
    pub progress_listener: Option<Arc<dyn UploadProgressListener>>,
}

impl MultipartUploadConfig {
    pub const DEFAULT_PART_SIZE: u64 = 5 * 1024 * 1024;
    pub const DEFAULT_CONCURRENCY: usize = 3;
    pub const DEFAULT_MAX_RETRIES: usize = 3;
    pub const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
    pub const DEFAULT_MULTIPART_THRESHOLD: u64 = 100 * 1024 * 1024;
}

impl_default!(MultipartUploadConfig => MultipartUploadConfig {
    part_size: MultipartUploadConfig::DEFAULT_PART_SIZE,
    concurrency: MultipartUploadConfig::DEFAULT_CONCURRENCY,
    max_retries: MultipartUploadConfig::DEFAULT_MAX_RETRIES,
    timeout_seconds: MultipartUploadConfig::DEFAULT_TIMEOUT_SECONDS,
    multipart_threshold: MultipartUploadConfig::DEFAULT_MULTIPART_THRESHOLD,
    progress_listener: None,
});

/// 单个分片的上传状态。
#[apply(plain_eq)]
pub struct PartInfo {
    /// 分片编号，遵循 S3 multipart upload 的 1-based 语义。
    pub part_number: u32,
    /// 分片在源文件中的起始字节偏移。
    pub start: u64,
    /// 分片在源文件中的结束字节偏移。
    pub end: u64,
    /// 分片大小，单位字节。
    pub size: u64,
    /// 上传成功后服务端返回的 ETag。
    pub etag: Option<String>,
    /// 当前分片状态。
    pub status: PartStatus,
}

/// 分片上传状态。
#[apply(plain_code_display_no_default_enum)]
pub enum PartStatus {
    /// 尚未开始上传。
    #[display("pending")]
    Pending,
    /// 正在上传。
    #[display("uploading")]
    Uploading,
    /// 已上传完成。
    #[display("completed")]
    Completed,
    /// 上传失败。
    #[display("failed")]
    Failed,
}

/// 一次分片上传任务的可持久化状态。
#[apply(plain_partial_eq)]
pub struct UploadStatus {
    /// S3 multipart upload id。
    pub upload_id: String,
    /// 目标 bucket。
    pub bucket_name: String,
    /// 目标对象 key。
    pub object_key: String,
    /// 文件总大小，单位字节。
    pub file_size: u64,
    /// 已上传字节数。
    pub uploaded_size: u64,
    /// 完成百分比。
    pub progress: f64,
    /// 所有分片状态。
    pub parts: Vec<PartInfo>,
    /// 上传任务状态。
    pub status: UploadStatusType,
    /// 失败时的错误信息。
    pub error: Option<String>,
    /// 创建时间，Unix epoch 毫秒。
    pub created_at_millis: u128,
    /// 最近更新时间，Unix epoch 毫秒。
    pub updated_at_millis: u128,
}

impl UploadStatus {
    /// 根据已上传字节数重新计算完成百分比。
    pub fn calculate_progress(&self) -> f64 {
        if self.file_size == 0 {
            return 0.0;
        }
        ((self.uploaded_size as f64 / self.file_size as f64) * 100.0).min(100.0)
    }
}

/// 分片上传任务状态。
#[apply(plain_code_display_no_default_enum)]
pub enum UploadStatusType {
    /// 已初始化，尚未正式上传。
    #[display("initialized")]
    Initialized,
    /// 上传中。
    #[display("in-progress")]
    InProgress,
    /// 上传完成。
    #[display("completed")]
    Completed,
    /// 上传失败。
    #[display("failed")]
    Failed,
    /// 上传已取消。
    #[display("cancelled")]
    Cancelled,
}

/// 分片上传流程返回结果。
#[apply(plain_partial_eq)]
pub enum MultipartUploadResult {
    /// 上传已成功完成。
    Success {
        bucket_name: String,
        object_key: String,
        upload_id: String,
        etag: String,
        file_size: u64,
        parts_count: usize,
    },
    /// 上传失败。
    Failed {
        bucket_name: String,
        object_key: String,
        upload_id: Option<String>,
        error: String,
    },
    /// 上传仍在进行，可根据状态继续跟踪。
    InProgress {
        upload_id: String,
        status: UploadStatus,
    },
}

/// 面向 UI 或日志展示的上传进度。
#[apply(plain_partial_eq)]
pub struct UploadProgress {
    /// 总字节数。
    pub total_bytes: u64,
    /// 已上传字节数。
    pub uploaded_bytes: u64,
    /// 完成百分比。
    pub percent: f64,
    /// 当前分片编号。
    pub current_part: Option<u32>,
    /// 总分片数。
    pub total_parts: Option<u32>,
    /// 当前估算上传速度，单位字节/秒。
    pub speed: Option<u64>,
    /// 估算剩余时间，单位秒。
    pub remaining_seconds: Option<u64>,
}

impl UploadProgress {
    /// 判断进度是否达到完成状态。
    pub fn is_complete(&self) -> bool {
        self.percent >= 100.0
    }

    /// 返回适合日志或简易终端展示的进度文本。
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

    /// 将字节数格式化为 `B`、`KB`、`MB` 或 `GB`。
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

    /// 将秒数格式化为紧凑时间文本。
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

/// 上传进度状态存储接口。
///
/// 该接口用于断点续传、进度查询和跨回调同步；返回 `bool` 表示对应状态是否存在或更新是否生效。
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

/// 内存上传进度状态存储，主要用于测试和单进程本地运行。
#[apply(plain_default_debug)]
pub struct InMemoryUploadProgressStorage {
    state: Mutex<HashMap<String, UploadStatus>>,
}

impl InMemoryUploadProgressStorage {
    /// 根据 bucket 和 object key 生成进度状态 key。
    pub fn generate_key(bucket_name: &str, object_key: &str) -> String {
        format!("upload:progress:{bucket_name}:{object_key}")
    }

    /// 根据 multipart upload id 生成状态 key。
    pub fn generate_upload_id_key(upload_id: &str) -> String {
        format!("upload:progress:id:{upload_id}")
    }
}

impl UploadProgressStorage for InMemoryUploadProgressStorage {
    fn save_status(&self, key: &str, status: UploadStatus) -> bool {
        recover_lock(&self.state).insert(key.to_owned(), status);
        true
    }

    fn get_status(&self, key: &str) -> Option<UploadStatus> {
        recover_lock(&self.state).get(key).cloned()
    }

    fn delete_status(&self, key: &str) -> bool {
        recover_lock(&self.state).remove(key).is_some()
    }

    fn update_part_status(
        &self,
        key: &str,
        part_number: u32,
        status: PartStatus,
        etag: Option<String>,
    ) -> bool {
        let mut state = recover_lock(&self.state);
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
        let mut state = recover_lock(&self.state);
        let Some(current) = state.get_mut(key) else {
            return false;
        };
        current.uploaded_size = uploaded_size;
        current.progress = current.calculate_progress();
        current.updated_at_millis = now_millis();
        true
    }
}

/// 带速度估算的上传进度监听器。
///
/// 它会把原始字节进度转换成 [`UploadProgress`]，并可选同步到 [`UploadProgressStorage`]。
pub struct SpeedTrackingProgressListener {
    progress_storage: Option<Arc<dyn UploadProgressStorage>>,
    bucket_name: Option<String>,
    object_key: Option<String>,
    on_update: Box<dyn Fn(UploadProgress) + Send + Sync>,
    last_sample: Mutex<Option<(Instant, u64)>>,
}

impl SpeedTrackingProgressListener {
    /// 创建速度追踪监听器。
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

    /// 重置速度采样窗口。
    pub fn reset(&self) {
        *recover_lock(&self.last_sample) = None;
    }
}

impl UploadProgressListener for SpeedTrackingProgressListener {
    fn on_progress(&self, progress: UploadProgressData) {
        let mut last_sample = recover_lock(&self.last_sample);
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

/// 返回当前 Unix epoch 毫秒时间戳。
pub fn now_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[allow(dead_code)]
/// 从对象元数据列表中提取 key 列表。
pub fn metadata_from_objects(objects: &[ObjectMetadata]) -> Vec<String> {
    objects.iter().map(|object| object.key.clone()).collect()
}
