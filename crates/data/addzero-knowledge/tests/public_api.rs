use std::{fs, path::PathBuf};

use addzero_knowledge::{KnowledgeSourceSpec, discover_documents, render_catalog};
use tempfile::tempdir;

#[test]
fn discovery_and_catalog_cover_markdown_sources() {
    let temp = tempdir().expect("tempdir should exist");
    let root = temp.path().join("notes");
    fs::create_dir_all(&root).expect("root should exist");
    fs::write(
        root.join("intro.md"),
        "# Hello\n\n## Section\n\nSome useful content.",
    )
    .expect("markdown should be written");

    let sources = vec![KnowledgeSourceSpec::new(
        "notes",
        "笔记",
        PathBuf::from(&root),
    )];
    let scan = discover_documents(&sources);

    assert_eq!(scan.documents.len(), 1);
    assert_eq!(scan.documents[0].title, "Hello");
    assert_eq!(scan.documents[0].headings, vec!["Section".to_string()]);

    let rendered = render_catalog("filesystem-fallback", &sources, &scan.documents);
    assert!(rendered.contains("KNOWLEDGE_DOCS"));
    assert!(rendered.contains("filesystem-fallback"));
}
