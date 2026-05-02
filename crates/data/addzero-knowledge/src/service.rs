use addzero_persistence::PersistenceContext;
use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::{
    discovery::discover_source_documents,
    repository::KnowledgeRepository,
    types::{
        KnowledgeDocument, KnowledgeError, KnowledgeSourceSpec, KnowledgeSyncReport,
        ManualKnowledgeDocumentInput,
    },
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

    pub async fn upsert_manual_document(
        &self,
        input: ManualKnowledgeDocumentInput,
    ) -> Result<KnowledgeDocument, KnowledgeError> {
        let source = KnowledgeSourceSpec::new(
            input.source_slug.clone(),
            input.source_name.clone(),
            input.source_root.clone(),
        );
        let source_id = self.repository.upsert_source(&source).await?;
        let existing = self.repository.source_by_slug(&input.source_slug).await?;
        let source_root = existing
            .map(|item| item.root_path)
            .unwrap_or_else(|| input.source_root.clone());

        let document = build_manual_document(&input, source_root);
        self.repository
            .upsert_document(source_id, &document)
            .await?;
        Ok(document)
    }
}

fn build_manual_document(
    input: &ManualKnowledgeDocumentInput,
    source_root: String,
) -> KnowledgeDocument {
    let body = input.body.trim().to_string();
    let title = if input.title.trim().is_empty() {
        "untitled".to_string()
    } else {
        input.title.trim().to_string()
    };
    let source_label = if input.source_label.trim().is_empty() {
        "未注明来源".to_string()
    } else {
        input.source_label.trim().to_string()
    };
    let tags = input
        .tags
        .iter()
        .map(|tag| tag.trim())
        .filter(|tag| !tag.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let headings = if tags.is_empty() {
        Vec::new()
    } else {
        tags.into_iter().take(10).collect::<Vec<_>>()
    };
    let content = format!("# {title}\n\n## 来源\n{source_label}\n\n## 内容\n{body}\n");
    let content_hash = compute_hash(&content);
    let slug = format!(
        "{}-{}-{}",
        input.source_slug,
        slugify(&input.relative_path),
        &content_hash[..8]
    );
    let cleaned = clean_text(&content);
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
        body: content,
        content_hash,
    }
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

fn slugify(value: &str) -> String {
    let normalized = deunicode::deunicode(value);
    let mut slug = String::new();
    let mut last_dash = false;

    for ch in normalized.chars() {
        let lowered = ch.to_ascii_lowercase();
        if lowered.is_ascii_alphanumeric() {
            slug.push(lowered);
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }

    slug.trim_matches('-').to_string()
}

fn clean_text(content: &str) -> String {
    let mut in_code_block = false;
    let mut lines = Vec::new();

    for raw in content.lines() {
        let line = raw.trim();
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block || line.is_empty() || line.starts_with('#') {
            continue;
        }
        lines.push(line);
    }

    lines.join(" ")
}

fn truncate_chars(text: &str, limit: usize) -> String {
    text.chars().take(limit).collect()
}
