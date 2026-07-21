use crate::client::{BlockingS3StorageClient, S3StorageClient, S3StorageClientFactory};
use crate::types::{RustfsConfig, S3ClientConfig};
use std::sync::Arc;

/// 构造 S3 兼容对象存储客户端的命名空间入口。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Rustfs;

impl Rustfs {
    /// 使用显式 S3 兼容配置创建存储客户端。
    pub fn storage_client(config: impl Into<S3ClientConfig>) -> Arc<dyn S3StorageClient> {
        create_storage_client(config)
    }

    /// 使用高层 RustFS 配置创建存储客户端。
    pub fn client(config: RustfsConfig) -> Arc<dyn S3StorageClient> {
        create_client(config)
    }

    /// 使用 [`RustfsConfig::default`] 创建默认存储客户端。
    pub fn default_client() -> Arc<dyn S3StorageClient> {
        create_default_client()
    }

    /// 通过注入的工厂创建存储客户端。
    pub fn storage_client_with_factory(
        factory: &dyn S3StorageClientFactory,
        config: impl Into<S3ClientConfig>,
    ) -> Arc<dyn S3StorageClient> {
        factory.create_client(config.into())
    }

    /// 创建工厂定义的默认存储客户端。
    pub fn default_client_with_factory(
        factory: &dyn S3StorageClientFactory,
    ) -> Arc<dyn S3StorageClient> {
        factory.create_default_client()
    }
}

/// 使用 S3 兼容配置创建存储客户端。
pub fn create_storage_client(config: impl Into<S3ClientConfig>) -> Arc<dyn S3StorageClient> {
    Arc::new(BlockingS3StorageClient::new(config.into()))
}

/// 使用 RustFS 配置创建存储客户端。
pub fn create_client(config: RustfsConfig) -> Arc<dyn S3StorageClient> {
    create_storage_client(config)
}

/// 创建默认本地存储客户端。
pub fn create_default_client() -> Arc<dyn S3StorageClient> {
    create_storage_client(RustfsConfig::default())
}
