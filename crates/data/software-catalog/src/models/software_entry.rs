use jiff::Timestamp;
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq, toasty::Model)]
#[table = "admin_software_entries"]
pub struct SoftwareEntryRecord {
    #[key]
    pub id: Uuid,
    #[index]
    pub slug: String,
    pub title: String,
    pub vendor: String,
    pub summary: String,
    pub homepage_url: String,
    pub icon_url: String,
    pub tags: toasty::Json<Vec<String>>,
    pub trial_platforms: toasty::Json<Vec<String>>,
    pub raw: toasty::Json<Value>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
