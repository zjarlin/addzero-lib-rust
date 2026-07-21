use jiff::Timestamp;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, toasty::Model)]
#[table = "knowledge_sources"]
pub struct KnowledgeSourceRecord {
    #[key]
    pub id: Uuid,
    #[index]
    pub slug: String,
    pub name: String,
    pub root_path: String,
    pub last_synced_at: Option<Timestamp>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
