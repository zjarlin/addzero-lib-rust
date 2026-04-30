use addzero_persistence::PersistenceContext;
use chrono::Utc;

use crate::{
    discovery::discover_source_documents,
    repository::KnowledgeRepository,
    types::{KnowledgeDocument, KnowledgeError, KnowledgeSourceSpec, KnowledgeSyncReport},
};

#[derive(Clone)]
pub struct KnowledgeService {
    repository: KnowledgeRepository,
}

impl KnowledgeService {
    pub async fn connect(database_url: &str) -> Result<Self, KnowledgeError> {
        let persistence = PersistenceContext::connect_with_url(database_url).await?;
        Ok(Self {
            repository: KnowledgeRepository::new(persistence.into_connection()),
        })
    }

    pub fn from_persistence(persistence: &PersistenceContext) -> Self {
        Self {
            repository: KnowledgeRepository::new(persistence.db().clone()),
        }
    }

    pub async fn list_documents(&self) -> Result<Vec<KnowledgeDocument>, KnowledgeError> {
        self.repository.list_documents().await
    }

    pub async fn sync_sources(
        &self,
        sources: &[KnowledgeSourceSpec],
    ) -> Result<KnowledgeSyncReport, KnowledgeError> {
        let mut report = KnowledgeSyncReport::default();

        for source in sources {
            if !source.root_path.exists() {
                continue;
            }

            let scan = discover_source_documents(source);
            let source_id = self.repository.upsert_source(source).await?;
            let mut active_paths = Vec::with_capacity(scan.documents.len());

            for doc in &scan.documents {
                self.repository.upsert_document(source_id, doc).await?;
                active_paths.push(doc.source_path.clone());
            }

            self.repository
                .deactivate_missing_documents(source_id, &active_paths)
                .await?;

            report.synced_sources.push(source.name.clone());
            report.upserted_documents += active_paths.len();
            report.skipped_paths.extend(scan.skipped_paths);
        }

        report.finished_at = Some(Utc::now());
        Ok(report)
    }
}
