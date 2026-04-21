use addzero_minio::*;
use addzero_rustfs::InMemoryS3StorageClient;
use std::fs;
use std::sync::Arc;
use tempfile::NamedTempFile;

fn test_client() -> MinioClient {
    let config = MinioConfig::builder("http://localhost:9000", "minioadmin", "minioadmin")
        .region("us-east-1")
        .build()
        .expect("config should build");
    MinioClient::from_storage_client(config, Arc::new(InMemoryS3StorageClient::default()))
        .expect("client should build")
}

#[test]
fn config_builder_matches_minio_defaults() {
    let config = MinioConfig::builder("http://localhost:9000", "minioadmin", "minioadmin")
        .build()
        .expect("config should build");

    assert_eq!(config.endpoint, "http://localhost:9000");
    assert_eq!(config.access_key, "minioadmin");
    assert_eq!(config.secret_key, "minioadmin");
    assert_eq!(config.region, None);
    assert!(config.path_style_access);
}

#[test]
fn bucket_and_object_lifecycle_matches_jvm_api() {
    let client = test_client();
    ensure_bucket(&client, "demo").expect("bucket should be ensured");

    let payload = b"hello minio";
    put_object(&client, "demo", "hello.txt", payload, Some("text/plain"))
        .expect("object should upload");

    assert!(bucket_exists(&client, "demo").expect("bucket check should work"));
    assert!(object_exists(&client, "demo", "hello.txt").expect("object check should work"));
    assert_eq!(
        get_object(&client, "demo", "hello.txt").expect("download should work"),
        payload
    );

    let info = stat_object(&client, "demo", "hello.txt")
        .expect("stat should work")
        .expect("object should exist");
    assert_eq!(info.object_name, "hello.txt");
    assert_eq!(info.size, payload.len() as u64);
    assert_eq!(info.content_type.as_deref(), Some("text/plain"));

    copy_object(&client, "demo", "hello.txt", "demo", "copy.txt").expect("copy should work");
    delete_object(&client, "demo", "hello.txt").expect("delete should work");

    let objects = list_objects(&client, "demo", None, true).expect("list should work");
    assert_eq!(objects.len(), 1);
    assert_eq!(objects[0].object_name, "copy.txt");
}

#[test]
fn file_upload_and_presigned_url_are_supported() {
    let client = test_client();
    ensure_bucket(&client, "demo").expect("bucket should be ensured");

    let file = NamedTempFile::new().expect("tempfile should exist");
    fs::write(file.path(), "{\"ok\":true}").expect("tempfile should be written");

    put_object_file(&client, "demo", "payload.json", file.path(), None)
        .expect("file upload should work");

    let info = stat_object(&client, "demo", "payload.json")
        .expect("stat should work")
        .expect("object should exist");
    assert_eq!(info.content_type.as_deref(), Some("application/json"));

    let download = NamedTempFile::new().expect("download tempfile should exist");
    get_object_to_file(&client, "demo", "payload.json", download.path())
        .expect("download should work");
    assert_eq!(
        fs::read_to_string(download.path()).expect("download should be readable"),
        "{\"ok\":true}"
    );

    let url = get_presigned_object_url(&client, "demo", "payload.json")
        .expect("presigned url should be created");
    assert!(url.contains("demo"));
    assert!(url.contains("payload.json"));
}

#[test]
fn cached_clients_reuse_the_same_key() {
    let config = MinioConfig::new("http://localhost:9000", "minioadmin", "minioadmin");
    let first = get_or_create_client("default", config.clone()).expect("first client should build");
    let second = get_or_create_client("default", config).expect("second client should reuse");

    assert_eq!(first.config(), second.config());
}
