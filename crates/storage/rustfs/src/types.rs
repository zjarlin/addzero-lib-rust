use std::collections::BTreeMap;
use std::time::SystemTime;


/// S3 兼容对象存储客户端配置。
///
/// `Debug` 输出会隐藏访问密钥和密钥内容，避免日志泄漏凭证。
#[derive(Clone, derive_more::Debug, Eq, PartialEq)]
pub struct S3ClientConfig {
    /// S3 兼容服务地址，例如 `http://localhost:9000`。
    pub endpoint: String,
    /// 访问密钥 ID，调试输出中会被隐藏。
    #[debug(skip)]
    pub access_key: String,
    /// 访问密钥 Secret，调试输出中会被隐藏。
    #[debug(skip)]
    pub secret_key: String,
    /// 签名区域；S3 兼容实现通常可使用 `us-east-1`。
    pub region: String,
    /// 是否使用 path-style bucket 地址，适合 MinIO/RustFS 等本地服务。
    pub path_style_access: bool,
}

impl S3ClientConfig {
    /// 使用 endpoint 和访问凭证创建配置。
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

    /// 设置签名区域。
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = region.into();
        self
    }

    /// 设置是否使用 path-style bucket 地址。
    pub fn with_path_style_access(mut self, enabled: bool) -> Self {
        self.path_style_access = enabled;
        self
    }
}

/// 对象元数据摘要。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectMetadata {
    /// 对象 key。
    pub key: String,
    /// 对象大小，单位字节。
    pub size: u64,
    /// 服务端返回的 ETag。
    pub etag: Option<String>,
    /// 服务端返回的最后修改时间字符串。
    pub last_modified: Option<String>,
    /// 对象内容类型。
    pub content_type: Option<String>,
    /// 用户自定义元数据。
    pub metadata: BTreeMap<String, String>,
}

/// 预签名访问地址。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresignedUrl {
    /// 可直接访问的预签名 URL。
    pub url: String,
    /// URL 过期时间。
    pub expiration: SystemTime,
}

/// RustFS 本地默认配置。
///
/// 该类型用于更高层入口，最终会转换为 [`S3ClientConfig`]。调试输出同样会隐藏凭证。
#[derive(Clone, derive_more::Debug, Eq, PartialEq)]
pub struct RustfsConfig {
    /// RustFS/S3 兼容服务地址。
    pub endpoint: String,
    /// 访问密钥 ID，调试输出中会被隐藏。
    #[debug(skip)]
    pub access_key: String,
    /// 访问密钥 Secret，调试输出中会被隐藏。
    #[debug(skip)]
    pub secret_key: String,
    /// 签名区域。
    pub region: String,
}

impl RustfsConfig {
    /// 返回面向本地 RustFS/MinIO 开发环境的默认配置。
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
        RustfsConfig::default_local()
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
