use std::{collections::BTreeMap, fmt::Write as _};

use crate::{KnowledgeSourceSpec, types::KnowledgeDocument};
use az_derive_aliases::{apply, plain_eq};

macro_rules! push_catalog_line {
    ($output:expr, $($arg:tt)*) => {
        let _ = writeln!($output, $($arg)*);
    };
}

pub fn render_catalog(
    mode: &str,
    sources: &[KnowledgeSourceSpec],
    docs: &[KnowledgeDocument],
) -> String {
    let mut output = String::new();
    let summaries = build_source_summaries(sources, docs);

    push_catalog_line!(
        output,
        "pub const KNOWLEDGE_SOURCE_AVAILABLE: bool = {};",
        if docs.is_empty() { "false" } else { "true" }
    );
    push_catalog_line!(
        output,
        "pub const KNOWLEDGE_DATA_MODE: &str = {};",
        quote(mode)
    );
    push_catalog_line!(output, "pub const KNOWLEDGE_DOCS: &[KnowledgeDoc] = &[");

    for doc in docs {
        push_catalog_line!(output, "    KnowledgeDoc {{");
        push_catalog_line!(output, "        source_slug: {},", quote(&doc.source_slug));
        push_catalog_line!(output, "        source_name: {},", quote(&doc.source_name));
        push_catalog_line!(output, "        source_root: {},", quote(&doc.source_root));
        push_catalog_line!(output, "        slug: {},", quote(&doc.slug));
        push_catalog_line!(output, "        title: {},", quote(&doc.title));
        push_catalog_line!(output, "        filename: {},", quote(&doc.filename));
        push_catalog_line!(output, "        source_path: {},", quote(&doc.source_path));
        push_catalog_line!(
            output,
            "        relative_path: {},",
            quote(&doc.relative_path)
        );
        push_catalog_line!(output, "        bytes: {},", doc.bytes);
        push_catalog_line!(output, "        section_count: {},", doc.section_count);
        push_catalog_line!(output, "        preview: {},", quote(&doc.preview));
        push_catalog_line!(output, "        excerpt: {},", quote(&doc.excerpt));
        push_catalog_line!(output, "        headings: &[");
        for heading in &doc.headings {
            push_catalog_line!(output, "            {},", quote(heading));
        }
        push_catalog_line!(output, "        ],");
        push_catalog_line!(output, "    }},");
    }
    push_catalog_line!(output, "];");

    push_catalog_line!(
        output,
        "pub const KNOWLEDGE_SOURCE_SUMMARIES: &[KnowledgeSourceSummary] = &["
    );
    for summary in summaries {
        push_catalog_line!(output, "    KnowledgeSourceSummary {{");
        push_catalog_line!(output, "        slug: {},", quote(&summary.slug));
        push_catalog_line!(output, "        label: {},", quote(&summary.label));
        push_catalog_line!(output, "        root: {},", quote(&summary.root));
        push_catalog_line!(output, "        count: {},", summary.count);
        push_catalog_line!(output, "    }},");
    }
    push_catalog_line!(output, "];");

    output
}

fn build_source_summaries(
    sources: &[KnowledgeSourceSpec],
    docs: &[KnowledgeDocument],
) -> Vec<RenderedSourceSummary> {
    let mut counts = BTreeMap::new();
    for doc in docs {
        *counts.entry(doc.source_slug.clone()).or_insert(0usize) += 1;
    }

    let mut summaries = sources
        .iter()
        .map(|source| RenderedSourceSummary {
            slug: source.slug.clone(),
            label: source.name.clone(),
            root: source.root_path.display().to_string(),
            count: counts.get(&source.slug).copied().unwrap_or_default(),
        })
        .collect::<Vec<_>>();
    summaries.sort_by(|left, right| left.label.cmp(&right.label));
    summaries
}

fn quote(value: &str) -> String {
    format!("{value:?}")
}

#[apply(plain_eq)]
struct RenderedSourceSummary {
    slug: String,
    label: String,
    root: String,
    count: usize,
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::types::KnowledgeDocument;

    #[test]
    fn catalog_keeps_source_counts() {
        let sources = vec![KnowledgeSourceSpec::new(
            "rust",
            "rust",
            PathBuf::from("/tmp/rust"),
        )];
        let docs = vec![KnowledgeDocument {
            source_slug: "rust".to_string(),
            source_name: "rust".to_string(),
            source_root: "/tmp/rust".to_string(),
            slug: "rust-book".to_string(),
            title: "Rust".to_string(),
            filename: "book.md".to_string(),
            source_path: "/tmp/rust/book.md".to_string(),
            relative_path: "book.md".to_string(),
            bytes: 10,
            section_count: 1,
            preview: "preview".to_string(),
            excerpt: "excerpt".to_string(),
            headings: vec!["h1".to_string()],
            tags: vec!["rust".to_string()],
            body: "body".to_string(),
            content_hash: "abc".to_string(),
            updated_at: chrono::Utc::now(),
        }];

        let rendered = render_catalog("postgres-sync", &sources, &docs);
        assert!(rendered.contains("KNOWLEDGE_SOURCE_SUMMARIES"));
        assert!(rendered.contains("count: 1"));
        assert!(rendered.contains("postgres-sync"));
    }
}
