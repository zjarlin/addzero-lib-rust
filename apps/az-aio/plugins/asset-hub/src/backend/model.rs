use serde::{Deserialize, Serialize};

pub const TABLE_NAME_PREFIX: &str = "biz_asset_hub_";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
pub struct AssetRecord {
    #[key]
    pub id: String,
    #[index]
    pub kind: String,
    pub title: String,
    pub status: String,
    pub source: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AssetSummary {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub status: String,
    pub source: String,
}

impl From<AssetRecord> for AssetSummary {
    fn from(record: AssetRecord) -> Self {
        Self {
            id: record.id,
            kind: record.kind,
            title: record.title,
            status: record.status,
            source: record.source,
        }
    }
}
