use jiff::Timestamp;
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, toasty::Model)]
#[table = "assets"]
pub struct AssetRecord {
    #[key]
    pub id: Uuid,
    #[index]
    pub kind: String,
    pub title: String,
    pub body: String,
    pub tags: toasty::Json<Vec<String>>,
    pub status: String,
    pub metadata: toasty::Json<Value>,
    #[index]
    pub content_hash: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
