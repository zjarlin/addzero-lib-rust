use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
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
    pub body: String,
    pub content_hash: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
