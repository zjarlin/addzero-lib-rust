use jiff::Timestamp;
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, toasty::Model)]
#[table = "asset_edges"]
pub struct AssetEdgeRecord {
    #[key]
    pub id: Uuid,
    #[index]
    pub source_asset_id: Uuid,
    #[index]
    pub target_asset_id: Uuid,
    pub relation: String,
    pub confidence: f64,
    pub metadata: toasty::Json<Value>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}
