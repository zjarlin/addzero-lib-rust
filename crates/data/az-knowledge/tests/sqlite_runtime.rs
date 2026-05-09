use az_knowledge::{KnowledgeService, ManualKnowledgeDocumentInput};
use tempfile::tempdir;

#[tokio::test]
async fn sqlite_service_persists_manual_documents() {
    let temp = tempdir().expect("tempdir should exist");
    let database_url = format!(
        "sqlite://{}",
        temp.path().join("knowledge.sqlite3").display()
    );
    let service = KnowledgeService::connect(&database_url)
        .await
        .expect("sqlite service should connect");

    let saved = service
        .upsert_manual_document(ManualKnowledgeDocumentInput {
            source_slug: "workspace-notes".to_string(),
            source_name: "笔记工作台".to_string(),
            source_root: "msc-aio://notes".to_string(),
            source_path: "msc-aio://notes/desktop-first-note.md".to_string(),
            relative_path: "desktop-first-note.md".to_string(),
            title: "桌面首启".to_string(),
            source_label: "笔记工作台".to_string(),
            body: "第一次启动跳过 PG，直接落到本机 SQLite。".to_string(),
            tags: vec!["桌面".to_string(), "sqlite".to_string()],
        })
        .await
        .expect("manual document should be saved");

    assert_eq!(saved.title, "桌面首启");
    assert_eq!(saved.tags, vec!["桌面".to_string(), "sqlite".to_string()]);

    let documents = service
        .list_documents()
        .await
        .expect("documents should load");
    assert_eq!(documents.len(), 1);
    assert_eq!(
        documents[0].source_path,
        "msc-aio://notes/desktop-first-note.md"
    );
    assert_eq!(
        documents[0].tags,
        vec!["桌面".to_string(), "sqlite".to_string()]
    );

    service
        .delete_document_by_source_path("msc-aio://notes/desktop-first-note.md")
        .await
        .expect("document should be deactivated");
    let documents = service
        .list_documents()
        .await
        .expect("documents should reload");
    assert!(documents.is_empty());
}
