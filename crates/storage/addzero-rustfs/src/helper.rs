use crate::client::{S3StorageClient, StorageError, StorageResult};
use crate::progress::{
    InMemoryUploadProgressStorage, MultipartUploadConfig, MultipartUploadResult, PartInfo,
    PartStatus, UploadProgress, UploadProgressData, UploadStatus, UploadStatusType, now_millis,
};
use crate::types::ObjectMetadata;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone, PartialEq)]
pub enum RustfsResult {
    Success {
        message: String,
        data: BTreeMap<String, String>,
    },
    Error {
        message: String,
    },
    InProgress {
        message: String,
        progress: UploadProgress,
        upload_id: Option<String>,
    },
}

pub fn guess_content_type(path: &Path) -> String {
    mime_guess::from_path(path)
        .first_or_octet_stream()
        .essence_str()
        .to_owned()
}

pub fn should_use_multipart_upload(file_size: u64, threshold: u64) -> bool {
    file_size >= threshold
}

pub fn calculate_optimal_part_size(file_size: u64) -> u64 {
    const ONE_HUNDRED_MB: u64 = 100 * 1024 * 1024;
    const ONE_GB: u64 = 1024 * 1024 * 1024;
    const TEN_GB: u64 = 10 * 1024 * 1024 * 1024;

    if file_size <= ONE_HUNDRED_MB {
        5 * 1024 * 1024
    } else if file_size <= ONE_GB {
        10 * 1024 * 1024
    } else if file_size <= TEN_GB {
        50 * 1024 * 1024
    } else {
        100 * 1024 * 1024
    }
}

pub fn generate_part_infos(file_size: u64, part_size: u64) -> Vec<PartInfo> {
    if file_size == 0 {
        return Vec::new();
    }

    let mut parts = Vec::new();
    let mut start = 0u64;
    let mut part_number = 1u32;

    while start < file_size {
        let size = (file_size - start).min(part_size);
        let end = start + size;
        parts.push(PartInfo {
            part_number,
            start,
            end,
            size,
            etag: None,
            status: PartStatus::Pending,
        });
        start = end;
        part_number += 1;
    }

    parts
}

pub fn smart_upload(
    client: Arc<dyn S3StorageClient>,
    bucket_name: &str,
    object_key: &str,
    file: &Path,
    config: MultipartUploadConfig,
    progress_storage: Option<Arc<dyn crate::progress::UploadProgressStorage>>,
) -> RustfsResult {
    let file_size = match std::fs::metadata(file) {
        Ok(metadata) => metadata.len(),
        Err(error) => {
            return RustfsResult::Error {
                message: error.to_string(),
            };
        }
    };

    if should_use_multipart_upload(file_size, config.multipart_threshold) {
        match upload_multipart(
            client,
            bucket_name,
            object_key,
            file,
            &config,
            progress_storage,
            None,
        ) {
            Ok(result) => match result {
                MultipartUploadResult::Success {
                    upload_id,
                    etag,
                    file_size,
                    parts_count,
                    ..
                } => RustfsResult::Success {
                    message: format!("File uploaded successfully: {bucket_name}/{object_key}"),
                    data: BTreeMap::from([
                        ("upload_id".to_owned(), upload_id),
                        ("etag".to_owned(), etag),
                        ("file_size".to_owned(), file_size.to_string()),
                        ("parts_count".to_owned(), parts_count.to_string()),
                    ]),
                },
                MultipartUploadResult::Failed { error, .. } => {
                    RustfsResult::Error { message: error }
                }
                MultipartUploadResult::InProgress { upload_id, status } => {
                    RustfsResult::InProgress {
                        message: "Upload in progress".to_owned(),
                        progress: UploadProgress {
                            total_bytes: status.file_size,
                            uploaded_bytes: status.uploaded_size,
                            percent: status.progress,
                            current_part: None,
                            total_parts: None,
                            speed: None,
                            remaining_seconds: None,
                        },
                        upload_id: Some(upload_id),
                    }
                }
            },
            Err(error) => RustfsResult::Error {
                message: error.to_string(),
            },
        }
    } else {
        let metadata = BTreeMap::new();
        match client.put_object_file(
            bucket_name,
            object_key,
            file,
            Some(&guess_content_type(file)),
            &metadata,
        ) {
            Ok(()) => RustfsResult::Success {
                message: format!("File uploaded successfully: {bucket_name}/{object_key}"),
                data: BTreeMap::from([("file_size".to_owned(), file_size.to_string())]),
            },
            Err(error) => RustfsResult::Error {
                message: error.to_string(),
            },
        }
    }
}

pub fn resume_or_upload(
    client: Arc<dyn S3StorageClient>,
    bucket_name: &str,
    object_key: &str,
    file: &Path,
    config: MultipartUploadConfig,
    progress_storage: Arc<dyn crate::progress::UploadProgressStorage>,
) -> RustfsResult {
    let storage_key = InMemoryUploadProgressStorage::generate_key(bucket_name, object_key);
    let existing = progress_storage.get_status(&storage_key);
    let existing_upload_id = existing.and_then(|status| {
        if matches!(
            status.status,
            UploadStatusType::Initialized | UploadStatusType::InProgress
        ) {
            Some(status.upload_id)
        } else {
            None
        }
    });

    match upload_multipart(
        client,
        bucket_name,
        object_key,
        file,
        &config,
        Some(progress_storage),
        existing_upload_id.as_deref(),
    ) {
        Ok(MultipartUploadResult::Success {
            upload_id,
            etag,
            file_size,
            parts_count,
            ..
        }) => RustfsResult::Success {
            message: format!("File uploaded successfully (resumed): {bucket_name}/{object_key}"),
            data: BTreeMap::from([
                ("upload_id".to_owned(), upload_id),
                ("etag".to_owned(), etag),
                ("file_size".to_owned(), file_size.to_string()),
                ("parts_count".to_owned(), parts_count.to_string()),
            ]),
        },
        Ok(MultipartUploadResult::InProgress { upload_id, status }) => RustfsResult::InProgress {
            message: "Upload in progress".to_owned(),
            progress: UploadProgress {
                total_bytes: status.file_size,
                uploaded_bytes: status.uploaded_size,
                percent: status.progress,
                current_part: None,
                total_parts: None,
                speed: None,
                remaining_seconds: None,
            },
            upload_id: Some(upload_id),
        },
        Ok(MultipartUploadResult::Failed { error, .. }) => RustfsResult::Error { message: error },
        Err(error) => RustfsResult::Error {
            message: error.to_string(),
        },
    }
}

pub fn upload_multipart(
    client: Arc<dyn S3StorageClient>,
    bucket_name: &str,
    object_key: &str,
    file: &Path,
    config: &MultipartUploadConfig,
    progress_storage: Option<Arc<dyn crate::progress::UploadProgressStorage>>,
    resume_upload_id: Option<&str>,
) -> StorageResult<MultipartUploadResult> {
    let file_size = std::fs::metadata(file)?.len();
    let part_size = config
        .part_size
        .max(MultipartUploadConfig::DEFAULT_PART_SIZE);
    let parts = generate_part_infos(file_size, part_size);
    if parts.is_empty() {
        return Err(StorageError::InvalidConfig(
            "cannot multipart upload an empty file".to_owned(),
        ));
    }

    let metadata = BTreeMap::new();
    let upload_id = match resume_upload_id {
        Some(upload_id) => upload_id.to_owned(),
        None => client.init_multipart_upload(
            bucket_name,
            object_key,
            Some(&guess_content_type(file)),
            &metadata,
        )?,
    };

    let storage_key = InMemoryUploadProgressStorage::generate_key(bucket_name, object_key);
    let initial_status = progress_storage
        .as_ref()
        .and_then(|storage| storage.get_status(&storage_key));
    let mut status = initial_status.unwrap_or_else(|| UploadStatus {
        upload_id: upload_id.clone(),
        bucket_name: bucket_name.to_owned(),
        object_key: object_key.to_owned(),
        file_size,
        uploaded_size: 0,
        progress: 0.0,
        parts: parts.clone(),
        status: UploadStatusType::Initialized,
        error: None,
        created_at_millis: now_millis(),
        updated_at_millis: now_millis(),
    });
    status.status = UploadStatusType::InProgress;
    if let Some(storage) = progress_storage.as_ref() {
        let _ = storage.save_status(&storage_key, status.clone());
    }

    let total_parts = parts.len() as u32;
    let batches = parts
        .into_iter()
        .filter(|part| {
            status
                .parts
                .iter()
                .find(|saved| saved.part_number == part.part_number)
                .map(|saved| saved.status != PartStatus::Completed)
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();

    let mut completed_parts = status.parts.clone();
    for chunk in batches.chunks(config.concurrency.max(1)) {
        let mut handles = Vec::new();
        for part in chunk.iter().cloned() {
            let client = Arc::clone(&client);
            let file_path = file.to_path_buf();
            let upload_id = upload_id.clone();
            let bucket = bucket_name.to_owned();
            let key = object_key.to_owned();
            let content_type = guess_content_type(file);
            let max_retries = config.max_retries;
            handles.push(thread::spawn(
                move || -> StorageResult<(u32, String, u64)> {
                    let bytes = read_part_bytes(&file_path, part.start, part.size)?;
                    let mut last_error = None;
                    for _ in 0..=max_retries {
                        match client.upload_part(
                            &bucket,
                            &key,
                            &upload_id,
                            part.part_number,
                            &bytes,
                            Some(&content_type),
                        ) {
                            Ok(etag) => return Ok((part.part_number, etag, part.size)),
                            Err(error) => last_error = Some(error),
                        }
                    }
                    Err(last_error.unwrap_or_else(|| {
                        StorageError::Backend(
                            "multipart upload failed without detailed error".to_owned(),
                        )
                    }))
                },
            ));
        }

        for handle in handles {
            let (part_number, etag, size) = handle
                .join()
                .map_err(|_| StorageError::Backend("multipart worker panicked".to_owned()))??;
            completed_parts = completed_parts
                .into_iter()
                .map(|part| {
                    if part.part_number == part_number {
                        PartInfo {
                            status: PartStatus::Completed,
                            etag: Some(etag.clone()),
                            ..part
                        }
                    } else {
                        part
                    }
                })
                .collect();

            let uploaded_size = completed_parts
                .iter()
                .filter(|part| part.status == PartStatus::Completed)
                .map(|part| part.size)
                .sum::<u64>();
            let progress = if file_size == 0 {
                0.0
            } else {
                (uploaded_size as f64 / file_size as f64) * 100.0
            };

            if let Some(listener) = config.progress_listener.as_ref() {
                listener.on_progress(UploadProgressData {
                    uploaded: uploaded_size,
                    total: file_size,
                    percent: progress,
                    part_number: Some(part_number),
                    total_parts: Some(total_parts),
                });
            }

            if let Some(storage) = progress_storage.as_ref() {
                let _ = storage.update_part_status(
                    &storage_key,
                    part_number,
                    PartStatus::Completed,
                    completed_parts
                        .iter()
                        .find(|part| part.part_number == part_number)
                        .and_then(|part| part.etag.clone()),
                );
                let _ = storage.update_uploaded_size(&storage_key, uploaded_size);
            }

            let _ = size;
        }
    }

    client.complete_multipart_upload(bucket_name, object_key, &upload_id, &completed_parts)?;
    if let Some(storage) = progress_storage.as_ref() {
        let final_status = UploadStatus {
            upload_id: upload_id.clone(),
            bucket_name: bucket_name.to_owned(),
            object_key: object_key.to_owned(),
            file_size,
            uploaded_size: file_size,
            progress: 100.0,
            parts: completed_parts.clone(),
            status: UploadStatusType::Completed,
            error: None,
            created_at_millis: status.created_at_millis,
            updated_at_millis: now_millis(),
        };
        let _ = storage.save_status(&storage_key, final_status);
        let _ = storage.delete_status(&InMemoryUploadProgressStorage::generate_upload_id_key(
            &upload_id,
        ));
    }

    let etag = completed_parts
        .iter()
        .filter_map(|part| part.etag.clone())
        .next_back()
        .unwrap_or_default();

    Ok(MultipartUploadResult::Success {
        bucket_name: bucket_name.to_owned(),
        object_key: object_key.to_owned(),
        upload_id,
        etag,
        file_size,
        parts_count: completed_parts.len(),
    })
}

pub fn build_list_request(
    bucket_name: &str,
    prefix: Option<&str>,
    recursive: bool,
    max_keys: usize,
) -> ListRequest {
    ListRequest {
        bucket_name: bucket_name.to_owned(),
        prefix: prefix.map(ToOwned::to_owned),
        delimiter: (!recursive).then(|| "/".to_owned()),
        max_keys,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListRequest {
    pub bucket_name: String,
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_keys: usize,
}

pub fn get_presigned_object_url(
    client: &dyn S3StorageClient,
    bucket_name: &str,
    key: &str,
    expires_in_seconds: u64,
) -> StorageResult<String> {
    client
        .generate_presigned_url(bucket_name, key, expires_in_seconds)
        .map(|url| url.url)
}

pub fn metadata_keys(objects: &[ObjectMetadata]) -> Vec<String> {
    objects.iter().map(|object| object.key.clone()).collect()
}

fn read_part_bytes(path: &Path, start: u64, size: u64) -> StorageResult<Vec<u8>> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(start))?;
    let mut buffer = vec![0u8; size as usize];
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}
