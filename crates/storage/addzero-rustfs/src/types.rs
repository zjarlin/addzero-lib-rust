use std::collections::BTreeMap;
use std::fmt;
use std::time::SystemTime;

#[derive(Clone, PartialEq, Eq)]
pub struct S3ClientConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub path_style_access: bool,
}

impl fmt::Debug for S3ClientConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("S3ClientConfig")
            .field("endpoint", &self.endpoint)
            .field("access_key", &"***")
            .field("secret_key", &"***")
            .field("region", &self.region)
            .field("path_style_access", &self.path_style_access)
            .finish()
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectMetadata {
    pub key: String,
    pub size: u64,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub content_type: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresignedUrl {
    pub url: String,
    pub expiration: SystemTime,
}

#[derive(Clone, PartialEq, Eq)]
pub struct RustfsConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
}

impl fmt::Debug for RustfsConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RustfsConfig")
            .field("endpoint", &self.endpoint)
            .field("access_key", &"***")
            .field("secret_key", &"***")
            .field("region", &self.region)
            .finish()
    }
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
mod tests {
    use super::*;

    #[test]
    fn s3_client_config_debug_masks_access_key_and_secret_key() {
        let config = S3ClientConfig::new("http://localhost:9000", "rustfs-access", "rustfs-secret");

        let debug = format!("{config:?}");

        assert!(debug.contains("access_key: \"***\""));
        assert!(debug.contains("secret_key: \"***\""));
        assert!(!debug.contains("rustfs-access"));
        assert!(!debug.contains("rustfs-secret"));
    }

    #[test]
    fn rustfs_config_debug_masks_access_key_and_secret_key() {
        let config = RustfsConfig::default_local();
        let debug = format!("{config:?}");

        assert!(debug.contains("access_key: \"***\""));
        assert!(debug.contains("secret_key: \"***\""));
        assert!(!debug.contains("rustfsadmin"));
    }
}
