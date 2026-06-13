#![cfg(not(target_arch = "wasm32"))]

use az_derive_aliases::{apply, plain_clone};
use az_persistence::PersistenceContext;

use crate::{
    import::{build_draft, fetch_metadata, seed_entries},
    model::{
        SoftwareCatalogDto, SoftwareCatalogError, SoftwareCatalogResult, SoftwareDraftInput,
        SoftwareEntryDto, SoftwareEntryInput, SoftwareMetadataDto, SoftwareMetadataFetchInput,
        current_platform,
    },
    repository::SoftwareCatalogRepository,
};

/// 软件目录应用服务。
///
/// 服务负责连接 PG 持久化、首次启动种子数据、目录查询、条目保存和主页元数据抓取。
#[apply(plain_clone)]
pub struct SoftwareCatalogService {
    repository: SoftwareCatalogRepository,
}

impl SoftwareCatalogService {
    /// 使用数据库 URL 连接持久化层并启动服务。
    pub async fn connect(database_url: &str) -> SoftwareCatalogResult<Self> {
        let persistence = PersistenceContext::connect_with_url(database_url)
            .await
            .map_err(SoftwareCatalogError::persistence)?;
        Self::boot(&persistence).await
    }

    /// 使用已存在的持久化上下文启动服务。
    ///
    /// 当目录为空时会写入内置默认软件条目。
    pub async fn boot(persistence: &PersistenceContext) -> SoftwareCatalogResult<Self> {
        let service = Self {
            repository: SoftwareCatalogRepository::new(persistence.db().clone()),
        };
        service.seed_defaults().await?;
        Ok(service)
    }

    /// 返回当前宿主平台和完整软件目录。
    pub async fn catalog(&self) -> SoftwareCatalogResult<SoftwareCatalogDto> {
        Ok(SoftwareCatalogDto {
            host_platform: current_platform(),
            items: self.repository.list_entries().await?,
        })
    }

    /// 创建或更新软件条目。
    ///
    /// 输入会在 repository 层完成 trim、去重、空方法过滤和缺失 ID 生成。
    pub async fn save_entry(
        &self,
        input: SoftwareEntryInput,
    ) -> SoftwareCatalogResult<SoftwareEntryDto> {
        self.repository.save_entry(input).await
    }

    /// 按条目 ID 删除软件。
    pub async fn delete_entry(&self, id: &str) -> SoftwareCatalogResult<()> {
        self.repository.delete_entry(id).await
    }

    /// 从软件主页抓取标题、摘要和图标等元数据。
    pub async fn fetch_metadata(
        &self,
        input: SoftwareMetadataFetchInput,
    ) -> SoftwareCatalogResult<SoftwareMetadataDto> {
        fetch_metadata(input).await
    }

    /// 根据主页和偏好平台构建可编辑的软件草稿。
    pub async fn build_draft(
        &self,
        input: SoftwareDraftInput,
    ) -> SoftwareCatalogResult<SoftwareEntryInput> {
        build_draft(input).await
    }

    async fn seed_defaults(&self) -> SoftwareCatalogResult<()> {
        if self.repository.count_entries().await? > 0 {
            return Ok(());
        }

        for input in seed_entries() {
            self.repository.save_entry(input).await?;
        }
        Ok(())
    }
}
