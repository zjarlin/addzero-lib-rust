use az_derive_aliases::{apply, seaorm_entity_model, seaorm_relation};
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[apply(seaorm_entity_model)]
#[sea_orm(table_name = "assets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
    pub status: String,
    pub metadata: Json,
    pub content_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[apply(seaorm_relation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
