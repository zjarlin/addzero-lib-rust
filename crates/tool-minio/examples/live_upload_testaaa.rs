use std::env;
use tool_minio::{DEFAULT_PRESIGNED_EXPIRATION_SECONDS, MinioClient, MinioConfig};

fn required_env(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    env::var(name).map_err(|_| format!("{name} is required").into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = required_env("MINIO_TEST_ENDPOINT")?;
    let access_key = required_env("MINIO_TEST_ACCESS_KEY")?;
    let secret_key = required_env("MINIO_TEST_SECRET_KEY")?;
    let bucket_name = required_env("MINIO_TEST_BUCKET")?;
    let encryption_secret = required_env("MINIO_URL_ENCRYPTION_SECRET")?;

    let config = MinioConfig::builder(endpoint, access_key, secret_key).build()?;
    let client = MinioClient::new(config)?;

    let buckets = client.list_buckets()?;
    println!("buckets={buckets:?}");

    let access = client.upload_text_and_encrypt_url(
        &bucket_name,
        "testaaa.txt",
        "hello from addzero-lib-rust",
        &encryption_secret,
        DEFAULT_PRESIGNED_EXPIRATION_SECONDS,
    )?;

    println!("relative_path={}", access.relative_path);
    println!("plain_url={}", access.plain_url);
    println!("encrypted_url={}", access.encrypted_url);
    Ok(())
}
