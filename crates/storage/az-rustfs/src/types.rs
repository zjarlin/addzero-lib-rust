use std::collections::BTreeMap;
use std::time::SystemTime;

use az_derive_aliases::{apply, plain_eq, plain_eq_no_debug};
use derive_more::Debug;

#[apply(plain_eq_no_debug)]
#[derive(Debug)]
pub struct S3ClientConfig {
    pub endpoint: String,
    #[debug(skip)]
    pub access_key: String,
    #[debug(skip)]
    pub secret_key: String,
    pub region: String,
    pub path_style_access: bool,
}

impl S3ClientConfig {
    pub fn new(
        endpoint: impl Into<String>,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            access_key: access_key.into(),
            secret_key: secret_key.into(),
            region: "us-east-1".to_owned(),
            path_style_access: true,
        }
    }

    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = region.into();
        self
    }

    pub fn with_path_style_access(mut self, enabled: bool) -> Self {
        self.path_style_access = enabled;
        self
    }
}

#[apply(plain_eq)]
pub struct ObjectMetadata {
    pub key: String,
    pub size: u64,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub content_type: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

#[apply(plain_eq)]
pub struct PresignedUrl {
    pub url: String,
    pub expiration: SystemTime,
}

#[apply(plain_eq_no_debug)]
#[derive(Debug)]
pub struct RustfsConfig {
    pub endpoint: String,
    #[debug(skip)]
    pub access_key: String,
    #[debug(skip)]
    pub secret_key: String,
    pub region: String,
}

impl RustfsConfig {
    pub fn default_local() -> Self {
        Self {
            endpoint: "http://localhost:9000".to_owned(),
            access_key: "rustfsadmin".to_owned(),
            secret_key: "rustfsadmin".to_owned(),
            region: "us-east-1".to_owned(),
        }
    }
}

impl Default for RustfsConfig {
    fn default() -> Self {
        Self::default_local()
    }
}

impl From<RustfsConfig> for S3ClientConfig {
    fn from(value: RustfsConfig) -> Self {
        S3ClientConfig::new(value.endpoint, value.access_key, value.secret_key)
            .with_region(value.region)
            .with_path_style_access(true)
    }
}

#[cfg(test)]
mod debug_redaction_tests {
    use super::{RustfsConfig, S3ClientConfig};

    #[test]
    fn s3_client_config_debug_does_not_leak_keys() {
        let config = S3ClientConfig::new("http://localhost:9000", "rustfsadmin", "rustfs-secret");

        let output = format!("{config:?}");
        assert!(output.contains("http://localhost:9000"));
        assert!(!output.contains("rustfsadmin"));
        assert!(!output.contains("rustfs-secret"));
    }

    #[test]
    fn rustfs_config_debug_does_not_leak_keys() {
        let config = RustfsConfig {
            endpoint: "http://localhost:9000".to_owned(),
            access_key: "rustfsadmin".to_owned(),
            secret_key: "rustfs-secret".to_owned(),
            region: "us-east-1".to_owned(),
        };

        let output = format!("{config:?}");
        assert!(output.contains("http://localhost:9000"));
        assert!(!output.contains("rustfsadmin"));
        assert!(!output.contains("rustfs-secret"));
    }
}
