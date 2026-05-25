use az_derive_aliases::{apply, seaorm_entity_model_eq, seaorm_relation};
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[apply(seaorm_entity_model_eq)]
#[sea_orm(table_name = "knowledge_documents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub source_id: Uuid,
    pub slug: String,
    pub title: String,
    pub filename: String,
    pub source_path: String,
    pub relative_path: String,
    pub bytes: i64,
    pub section_count: i32,
    pub preview: String,
    pub excerpt: String,
    pub headings: Vec<String>,
    pub tags: Vec<String>,
    pub body: String,
    pub content_hash: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[apply(seaorm_relation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
