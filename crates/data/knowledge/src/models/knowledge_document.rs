use jiff::Timestamp;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, toasty::Model)]
#[table = "knowledge_documents"]
pub struct KnowledgeDocumentRecord {
    #[key]
    pub id: Uuid,
    #[index]
    pub source_id: Uuid,
    #[index]
    pub slug: String,
    pub title: String,
    pub filename: String,
    #[index]
    pub source_path: String,
    pub relative_path: String,
    pub bytes: i64,
    pub section_count: i32,
    pub preview: String,
    pub excerpt: String,
    pub headings: toasty::Json<Vec<String>>,
    pub tags: toasty::Json<Vec<String>>,
    pub body: String,
    #[index]
    pub content_hash: String,
    pub is_active: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
