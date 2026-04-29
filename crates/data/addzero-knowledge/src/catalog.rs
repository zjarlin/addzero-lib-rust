use std::{collections::BTreeMap, fmt::Write as _};

use crate::{KnowledgeSourceSpec, types::KnowledgeDocument};

pub fn render_catalog(
    mode: &str,
    sources: &[KnowledgeSourceSpec],
    docs: &[KnowledgeDocument],
) -> String {
    let mut output = String::new();
    let summaries = build_source_summaries(sources, docs);

    writeln!(
        output,
        "pub const KNOWLEDGE_SOURCE_AVAILABLE: bool = {};",
        if docs.is_empty() { "false" } else { "true" }
    )
    .unwrap();
    writeln!(
        output,
        "pub const KNOWLEDGE_DATA_MODE: &str = {};",
        quote(mode)
    )
    .unwrap();
    writeln!(output, "pub const KNOWLEDGE_DOCS: &[KnowledgeDoc] = &[").unwrap();

    for doc in docs {
        writeln!(output, "    KnowledgeDoc {{").unwrap();
        writeln!(output, "        source_slug: {},", quote(&doc.source_slug)).unwrap();
        writeln!(output, "        source_name: {},", quote(&doc.source_name)).unwrap();
        writeln!(output, "        source_root: {},", quote(&doc.source_root)).unwrap();
        writeln!(output, "        slug: {},", quote(&doc.slug)).unwrap();
        writeln!(output, "        title: {},", quote(&doc.title)).unwrap();
        writeln!(output, "        filename: {},", quote(&doc.filename)).unwrap();
        writeln!(output, "        source_path: {},", quote(&doc.source_path)).unwrap();
        writeln!(
            output,
            "        relative_path: {},",
            quote(&doc.relative_path)
        )
        .unwrap();
        writeln!(output, "        bytes: {},", doc.bytes).unwrap();
        writeln!(output, "        section_count: {},", doc.section_count).unwrap();
        writeln!(output, "        preview: {},", quote(&doc.preview)).unwrap();
        writeln!(output, "        excerpt: {},", quote(&doc.excerpt)).unwrap();
        writeln!(output, "        headings: &[").unwrap();
        for heading in &doc.headings {
            writeln!(output, "            {},", quote(heading)).unwrap();
        }
        writeln!(output, "        ],").unwrap();
        writeln!(output, "    }},").unwrap();
    }
    writeln!(output, "];").unwrap();

    writeln!(
        output,
        "pub const KNOWLEDGE_SOURCE_SUMMARIES: &[KnowledgeSourceSummary] = &["
    )
    .unwrap();
    for summary in summaries {
        writeln!(output, "    KnowledgeSourceSummary {{").unwrap();
        writeln!(output, "        slug: {},", quote(&summary.slug)).unwrap();
        writeln!(output, "        label: {},", quote(&summary.label)).unwrap();
        writeln!(output, "        root: {},", quote(&summary.root)).unwrap();
        writeln!(output, "        count: {},", summary.count).unwrap();
        writeln!(output, "    }},").unwrap();
    }
    writeln!(output, "];").unwrap();

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

#[derive(Clone, Debug, PartialEq, Eq)]
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
            body: "body".to_string(),
            content_hash: "abc".to_string(),
        }];

        let rendered = render_catalog("postgres-sync", &sources, &docs);
        assert!(rendered.contains("KNOWLEDGE_SOURCE_SUMMARIES"));
        assert!(rendered.contains("count: 1"));
        assert!(rendered.contains("postgres-sync"));
    }
}
