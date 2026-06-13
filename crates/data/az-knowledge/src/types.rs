use std::path::PathBuf;

use az_derive_aliases::{apply, serde_eq, serde_eq_default};
use chrono::{DateTime, Utc};

#[apply(serde_eq)]
pub struct KnowledgeSourceSpec {
    pub slug: String,
    pub name: String,
    pub root_path: PathBuf,
}

impl KnowledgeSourceSpec {
    pub fn new(
        slug: impl Into<String>,
        name: impl Into<String>,
        root_path: impl Into<PathBuf>,
    ) -> Self {
        Self {
            slug: slug.into(),
            name: name.into(),
            root_path: root_path.into(),
        }
    }
}

#[apply(serde_eq)]
pub struct KnowledgeDocument {
    pub source_slug: String,
    pub source_name: String,
    pub source_root: String,
    pub slug: String,
    pub title: String,
    pub filename: String,
    pub source_path: String,
    pub relative_path: String,
    pub bytes: usize,
    pub section_count: usize,
    pub preview: String,
    pub excerpt: String,
    pub headings: Vec<String>,
    pub tags: Vec<String>,
    pub body: String,
    pub content_hash: String,
    pub updated_at: DateTime<Utc>,
}

#[apply(serde_eq)]
pub struct ManualKnowledgeDocumentInput {
    pub source_slug: String,
    pub source_name: String,
    pub source_root: String,
    pub source_path: String,
    pub relative_path: String,
    pub title: String,
    pub source_label: String,
    pub body: String,
    pub tags: Vec<String>,
}

#[apply(serde_eq_default)]
pub struct KnowledgeScan {
    pub documents: Vec<KnowledgeDocument>,
    pub skipped_paths: Vec<String>,
}

#[apply(serde_eq_default)]
pub struct KnowledgeSyncReport {
    pub synced_sources: Vec<String>,
    pub upserted_documents: usize,
    pub skipped_paths: Vec<String>,
    pub finished_at: Option<DateTime<Utc>>,
}
