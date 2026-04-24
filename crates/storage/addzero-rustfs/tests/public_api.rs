use addzero_rustfs::*;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;

#[test]
fn client_config_and_rustfs_defaults_match_expected_values() {
    let config = RustfsConfig::default();
    let s3_config: S3ClientConfig = config.clone().into();

    assert_eq!(config.endpoint, "http://localhost:9000");
    assert_eq!(config.access_key, "rustfsadmin");
    assert_eq!(config.secret_key, "rustfsadmin");
    assert_eq!(s3_config.region, "us-east-1");
    assert!(s3_config.path_style_access);
}

#[test]
fn in_memory_client_supports_bucket_and_object_lifecycle() {
    let client = InMemoryS3StorageClient::default();
    client.create_bucket("demo").expect("bucket should create");
    client
        .put_object_bytes(
            "demo",
            "hello.txt",
            b"hello",
            Some("text/plain"),
            &BTreeMap::new(),
        )
        .expect("object should upload");

    assert!(
        client
            .bucket_exists("demo")
            .expect("bucket check should work")
    );
    assert!(
        client
            .object_exists("demo", "hello.txt")
            .expect("object check should work")
    );
    assert_eq!(
        client
            .get_object("demo", "hello.txt")
            .expect("object should download"),
        b"hello"
    );

    let listed = client
        .list_objects("demo", Some("hello"), true, 100)
        .expect("list should work");
    assert_eq!(listed.len(), 1);

    client
        .copy_object("demo", "hello.txt", "demo", "copy.txt")
        .expect("copy should work");
    client
        .delete_object("demo", "hello.txt")
        .expect("delete should work");
    assert!(
        client
            .object_exists("demo", "copy.txt")
            .expect("copied object should exist")
    );
}

#[test]
fn multipart_upload_and_resume_helpers_work_with_in_memory_client() {
    let client: Arc<dyn S3StorageClient> = Arc::new(InMemoryS3StorageClient::default());
    client.create_bucket("demo").expect("bucket should create");

    let tempfile = NamedTempFile::new().expect("tempfile should exist");
    std::fs::write(tempfile.path(), vec![42u8; 6 * 1024 * 1024 + 64]).expect("file should write");

    let storage: Arc<dyn UploadProgressStorage> =
        Arc::new(InMemoryUploadProgressStorage::default());
    let updates = Arc::new(Mutex::new(Vec::new()));
    let updates_ref = Arc::clone(&updates);
    let listener = Arc::new(SpeedTrackingProgressListener::new(
        Some(Arc::clone(&storage)),
        Some("demo".to_owned()),
        Some("big.bin".to_owned()),
        move |progress| {
            updates_ref
                .lock()
                .expect("updates mutex should not be poisoned")
                .push(progress);
        },
    ));

    let config = MultipartUploadConfig {
        part_size: 5 * 1024 * 1024,
        concurrency: 2,
        max_retries: 1,
        timeout_seconds: 10,
        multipart_threshold: 1024,
        progress_listener: Some(listener),
    };

    let result = upload_multipart(
        Arc::clone(&client),
        "demo",
        "big.bin",
        tempfile.path(),
        &config,
        Some(storage),
        None,
    )
    .expect("multipart upload should succeed");

    match result {
        MultipartUploadResult::Success {
            upload_id,
            parts_count,
            ..
        } => {
            assert!(!upload_id.is_empty());
            assert_eq!(parts_count, 2);
        }
        other => panic!("expected success, got {other:?}"),
    }

    let object = client
        .get_object("demo", "big.bin")
        .expect("uploaded object should exist");
    assert_eq!(object.len(), 6 * 1024 * 1024 + 64);
    assert!(
        !updates
            .lock()
            .expect("updates mutex should not be poisoned")
            .is_empty()
    );
}

#[test]
fn progress_helpers_compute_expected_values() {
    assert!(should_use_multipart_upload(
        100 * 1024 * 1024,
        100 * 1024 * 1024
    ));
    assert_eq!(
        calculate_optimal_part_size(50 * 1024 * 1024),
        5 * 1024 * 1024
    );

    let parts = generate_part_infos(11, 5);
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0].size, 5);
    assert_eq!(parts[2].size, 1);

    let progress = UploadProgress {
        total_bytes: 10 * 1024 * 1024,
        uploaded_bytes: 5 * 1024 * 1024,
        percent: 50.0,
        current_part: Some(2),
        total_parts: Some(4),
        speed: Some(1024 * 1024),
        remaining_seconds: Some(5),
    };
    assert!(progress.formatted().contains("50.00%"));
    assert!(progress.formatted().contains("(2/4)"));
    assert!(!progress.is_complete());
}

#[test]
fn speed_tracking_listener_calculates_speed_and_updates_storage() {
    let storage: Arc<dyn UploadProgressStorage> =
        Arc::new(InMemoryUploadProgressStorage::default());
    let key = InMemoryUploadProgressStorage::generate_key("bucket", "key");
    let status = UploadStatus {
        upload_id: "upload-1".to_owned(),
        bucket_name: "bucket".to_owned(),
        object_key: "key".to_owned(),
        file_size: 2048,
        uploaded_size: 0,
        progress: 0.0,
        parts: Vec::new(),
        status: UploadStatusType::InProgress,
        error: None,
        created_at_millis: 0,
        updated_at_millis: 0,
    };
    let _ = storage.save_status(&key, status);

    let values = Arc::new(Mutex::new(Vec::new()));
    let values_ref = Arc::clone(&values);
    let listener = SpeedTrackingProgressListener::new(
        Some(Arc::clone(&storage)),
        Some("bucket".to_owned()),
        Some("key".to_owned()),
        move |progress| {
            values_ref
                .lock()
                .expect("values mutex should not be poisoned")
                .push(progress);
        },
    );

    listener.on_progress(UploadProgressData {
        uploaded: 512,
        total: 1024,
        percent: 50.0,
        part_number: Some(1),
        total_parts: Some(2),
    });
    thread::sleep(Duration::from_millis(10));
    listener.on_progress(UploadProgressData {
        uploaded: 1024,
        total: 1024,
        percent: 100.0,
        part_number: Some(2),
        total_parts: Some(2),
    });

    let recorded = values.lock().expect("values mutex should not be poisoned");
    assert_eq!(recorded.len(), 2);
    assert!(recorded[1].speed.is_some());
    let updated = storage
        .get_status(&key)
        .expect("status should be present after updates");
    assert_eq!(updated.uploaded_size, 1024);
}

#[test]
fn build_list_request_and_presigned_helpers_match_expected_shape() {
    let request = build_list_request("bucket", Some("data/"), false, 1000);
    assert_eq!(request.bucket_name, "bucket");
    assert_eq!(request.prefix.as_deref(), Some("data/"));
    assert_eq!(request.delimiter.as_deref(), Some("/"));
    assert_eq!(request.max_keys, 1000);

    let client = InMemoryS3StorageClient::default();
    let url = get_presigned_object_url(&client, "bucket", "hello.txt", 60)
        .expect("presigned url should generate");
    assert!(url.contains("bucket"));
    assert!(url.contains("hello.txt"));
}
