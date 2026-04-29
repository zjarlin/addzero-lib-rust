use chrono::Utc;

use crate::{
    discovery::discover_source_documents,
    pg_repo::PgRepo,
    types::{KnowledgeError, KnowledgeSourceSpec, KnowledgeSyncReport},
};

#[derive(Clone)]
pub struct KnowledgeService {
    pg: PgRepo,
}

impl KnowledgeService {
    pub async fn connect(database_url: &str) -> Result<Self, KnowledgeError> {
        let pg = PgRepo::connect(database_url).await?;
        pg.ensure_schema().await?;
        Ok(Self { pg })
    }

    pub fn repo(&self) -> &PgRepo {
        &self.pg
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
            let source_id = self.pg.upsert_source(source).await?;
            let mut active_paths = Vec::with_capacity(scan.documents.len());

            for doc in &scan.documents {
                self.pg.upsert_document(source_id, doc).await?;
                active_paths.push(doc.source_path.clone());
            }

            self.pg
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
