use az_derive_aliases::{apply, seaorm_entity_model, seaorm_relation};
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[apply(seaorm_entity_model)]
#[sea_orm(table_name = "asset_edges")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub source_asset_id: Uuid,
    pub target_asset_id: Uuid,
    pub relation: String,
    pub confidence: f64,
    pub metadata: Json,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[apply(seaorm_relation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
