#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KnowledgeDoc {
    pub source_slug: &'static str,
    pub source_name: &'static str,
    pub source_root: &'static str,
    pub slug: &'static str,
    pub title: &'static str,
    pub filename: &'static str,
    pub source_path: &'static str,
    pub relative_path: &'static str,
    pub bytes: usize,
    pub section_count: usize,
    pub preview: &'static str,
    pub excerpt: &'static str,
    pub headings: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KnowledgeSourceSummary {
    pub slug: &'static str,
    pub label: &'static str,
    pub root: &'static str,
    pub count: usize,
}

include!(concat!(env!("OUT_DIR"), "/knowledge_catalog.rs"));

pub fn knowledge_doc(slug: &str) -> Option<&'static KnowledgeDoc> {
    KNOWLEDGE_DOCS.iter().find(|doc| doc.slug == slug)
}

pub fn total_sections() -> usize {
    KNOWLEDGE_DOCS.iter().map(|doc| doc.section_count).sum()
}

pub fn total_bytes() -> usize {
    KNOWLEDGE_DOCS.iter().map(|doc| doc.bytes).sum()
}

pub fn total_sources() -> usize {
    KNOWLEDGE_SOURCE_SUMMARIES
        .iter()
        .filter(|summary| summary.count > 0)
        .count()
}
