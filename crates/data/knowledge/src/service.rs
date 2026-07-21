use az_persistence::context::PersistenceContext;
use az_str::{
    transformation::{MarkdownListMarkerMode, clean_markdown_plain_text, truncate_chars},
    sanitize::to_slug,
};
use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::{
    discovery::discover_source_documents,
    models::knowledge_models,
    repository::KnowledgeRepository,
    sqlite_repository::SqliteKnowledgeRepository,
    types::{
        KnowledgeDocument, KnowledgeSourceSpec, KnowledgeSyncReport, ManualKnowledgeDocumentInput,
    },
};

#[derive(Clone)]
pub struct KnowledgeService {
    backend: KnowledgeBackend,
}

#[derive(Clone)]
enum KnowledgeBackend {
    Postgres(KnowledgeRepository),
    Sqlite(SqliteKnowledgeRepository),
}

impl KnowledgeService {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        if database_url.starts_with("sqlite:") {
            return Ok(Self {
                backend: KnowledgeBackend::Sqlite(
                    SqliteKnowledgeRepository::connect(database_url).await?,
                ),
            });
        }

        let persistence =
            PersistenceContext::connect_with_url(database_url, knowledge_models()).await?;
        Ok(Self::from_persistence(&persistence))
    }

    pub fn from_persistence(persistence: &PersistenceContext) -> Self {
        Self {
            backend: KnowledgeBackend::Postgres(KnowledgeRepository::new(persistence.db().clone())),
        }
    }

    pub async fn list_documents(&self) -> anyhow::Result<Vec<KnowledgeDocument>> {
        match &self.backend {
            KnowledgeBackend::Postgres(repository) => repository.list_documents().await,
            KnowledgeBackend::Sqlite(repository) => repository.list_documents().await,
        }
    }

    pub async fn sync_sources(
        &self,
        sources: &[KnowledgeSourceSpec],
    ) -> anyhow::Result<KnowledgeSyncReport> {
        let mut report = KnowledgeSyncReport::default();

        match &self.backend {
            KnowledgeBackend::Postgres(repository) => {
                for source in sources {
                    if !source.root_path.exists() {
                        continue;
                    }

                    let scan = discover_source_documents(source);
                    let source_id = repository.upsert_source(source).await?;
                    let mut active_paths = Vec::with_capacity(scan.documents.len());

                    for doc in &scan.documents {
                        repository.upsert_document(source_id, doc).await?;
                        active_paths.push(doc.source_path.clone());
                    }

                    repository
                        .deactivate_missing_documents(source_id, &active_paths)
                        .await?;

                    report.synced_sources.push(source.name.clone());
                    report.upserted_documents += active_paths.len();
                    report.skipped_paths.extend(scan.skipped_paths);
                }
            }
            KnowledgeBackend::Sqlite(repository) => {
                for source in sources {
                    if !source.root_path.exists() {
                        continue;
                    }

                    let scan = discover_source_documents(source);
                    let source_id = repository.upsert_source(source).await?;
                    let mut active_paths = Vec::with_capacity(scan.documents.len());

                    for doc in &scan.documents {
                        repository.upsert_document(&source_id, doc).await?;
                        active_paths.push(doc.source_path.clone());
                    }

                    repository
                        .deactivate_missing_documents(&source_id, &active_paths)
                        .await?;

                    report.synced_sources.push(source.name.clone());
                    report.upserted_documents += active_paths.len();
                    report.skipped_paths.extend(scan.skipped_paths);
                }
            }
        }

        report.finished_at = Some(Utc::now());
        Ok(report)
    }

    pub async fn upsert_manual_document(
        &self,
        input: ManualKnowledgeDocumentInput,
    ) -> anyhow::Result<KnowledgeDocument> {
        let source = KnowledgeSourceSpec::new(
            input.source_slug.clone(),
            input.source_name.clone(),
            input.source_root.clone(),
        );

        match &self.backend {
            KnowledgeBackend::Postgres(repository) => {
                let source_id = repository.upsert_source(&source).await?;
                let existing = repository.source_by_slug(&input.source_slug).await?;
                let source_root = existing
                    .map(|item| item.root_path)
                    .unwrap_or_else(|| input.source_root.clone());
                let document = build_manual_document(&input, source_root);
                repository.upsert_document(source_id, &document).await?;
                Ok(document)
            }
            KnowledgeBackend::Sqlite(repository) => {
                let source_id = repository.upsert_source(&source).await?;
                let source_root = repository
                    .source_root_by_slug(&input.source_slug)
                    .await?
                    .unwrap_or_else(|| input.source_root.clone());
                let document = build_manual_document(&input, source_root);
                repository.upsert_document(&source_id, &document).await?;
                Ok(document)
            }
        }
    }

    pub async fn delete_document_by_source_path(
        &self,
        source_path: &str,
    ) -> anyhow::Result<()> {
        match &self.backend {
            KnowledgeBackend::Postgres(repository) => {
                repository
                    .deactivate_document_by_source_path(source_path)
                    .await
            }
            KnowledgeBackend::Sqlite(repository) => {
                repository
                    .deactivate_document_by_source_path(source_path)
                    .await
            }
        }
    }
}

fn build_manual_document(
    input: &ManualKnowledgeDocumentInput,
    source_root: String,
) -> KnowledgeDocument {
    let now = Utc::now();
    let title = derive_title(&input.title, &input.body);
    let content = normalize_manual_markdown(&title, &input.body);
    let headings = extract_headings(&content);
    let content_hash = compute_hash(&content);
    let slug = format!(
        "{}-{}-{}",
        input.source_slug,
        to_slug(&input.relative_path),
        &content_hash[..8]
    );
    let cleaned = clean_markdown_plain_text(&content, MarkdownListMarkerMode::Keep);
    KnowledgeDocument {
        source_slug: input.source_slug.clone(),
        source_name: input.source_name.clone(),
        source_root,
        slug,
        title,
        filename: filename_from_path(&input.relative_path),
        source_path: input.source_path.clone(),
        relative_path: input.relative_path.clone(),
        bytes: content.len(),
        section_count: headings.len(),
        preview: truncate_chars(&cleaned, 110),
        excerpt: truncate_chars(&cleaned, 900),
        headings,
        tags: normalize_tags(&input.tags),
        body: content,
        content_hash,
        updated_at: now,
    }
}

fn normalize_tags(raw: &[String]) -> Vec<String> {
    let mut tags = Vec::new();

    for tag in raw {
        let cleaned = tag.trim().trim_start_matches('#').trim();
        if cleaned.is_empty() {
            continue;
        }
        if tags.iter().any(|existing| existing == cleaned) {
            continue;
        }
        tags.push(cleaned.to_string());
    }

    tags
}

fn derive_title(title: &str, body: &str) -> String {
    let trimmed = title.trim();
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }

    if let Some(heading) = body
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with('#') && !line.trim_start_matches('#').trim().is_empty())
    {
        return heading.trim_start_matches('#').trim().to_string();
    }

    if let Some(line) = body.lines().map(str::trim).find(|line| !line.is_empty()) {
        return line
            .trim_start_matches(['#', '-', '*', ' '])
            .chars()
            .take(48)
            .collect::<String>();
    }

    "untitled".to_string()
}

fn normalize_manual_markdown(title: &str, body: &str) -> String {
    let trimmed = body.trim();
    let mut content = if trimmed.is_empty() {
        format!("# {title}\n\n")
    } else if trimmed
        .lines()
        .next()
        .is_some_and(|line| line.trim_start().starts_with('#'))
    {
        trimmed.to_string()
    } else {
        format!("# {title}\n\n{trimmed}")
    };

    if !content.ends_with('\n') {
        content.push('\n');
    }

    content
}

fn extract_headings(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with('#'))
        .filter_map(|line| {
            let heading = line.trim_start_matches('#').trim();
            (!heading.is_empty()).then(|| heading.to_string())
        })
        .take(12)
        .collect()
}

fn filename_from_path(relative_path: &str) -> String {
    relative_path
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or("untitled.md")
        .to_string()
}

fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
