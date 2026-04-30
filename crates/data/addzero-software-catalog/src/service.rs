use addzero_persistence::PersistenceContext;

use crate::{
    import::{build_draft, fetch_metadata, seed_entries},
    model::{
        SoftwareCatalogDto, SoftwareCatalogError, SoftwareCatalogResult, SoftwareDraftInput,
        SoftwareEntryDto, SoftwareEntryInput, SoftwareMetadataDto, SoftwareMetadataFetchInput,
        current_platform,
    },
    repository::SoftwareCatalogRepository,
};

#[derive(Clone)]
pub struct SoftwareCatalogService {
    repository: SoftwareCatalogRepository,
}

impl SoftwareCatalogService {
    pub async fn connect(database_url: &str) -> SoftwareCatalogResult<Self> {
        let persistence = PersistenceContext::connect_with_url(database_url)
            .await
            .map_err(SoftwareCatalogError::persistence)?;
        Self::boot(&persistence).await
    }

    pub async fn boot(persistence: &PersistenceContext) -> SoftwareCatalogResult<Self> {
        let service = Self {
            repository: SoftwareCatalogRepository::new(persistence.db().clone()),
        };
        service.seed_defaults().await?;
        Ok(service)
    }

    pub async fn catalog(&self) -> SoftwareCatalogResult<SoftwareCatalogDto> {
        Ok(SoftwareCatalogDto {
            host_platform: current_platform(),
            items: self.repository.list_entries().await?,
        })
    }

    pub async fn save_entry(
        &self,
        input: SoftwareEntryInput,
    ) -> SoftwareCatalogResult<SoftwareEntryDto> {
        self.repository.save_entry(input).await
    }

    pub async fn delete_entry(&self, id: &str) -> SoftwareCatalogResult<()> {
        self.repository.delete_entry(id).await
    }

    pub async fn fetch_metadata(
        &self,
        input: SoftwareMetadataFetchInput,
    ) -> SoftwareCatalogResult<SoftwareMetadataDto> {
        fetch_metadata(input).await
    }

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
