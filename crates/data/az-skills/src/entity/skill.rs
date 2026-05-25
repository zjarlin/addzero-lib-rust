use az_derive_aliases::{apply, seaorm_entity_model_eq, seaorm_relation};
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[apply(seaorm_entity_model_eq)]
#[sea_orm(table_name = "skills")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    pub keywords: Vec<String>,
    pub description: String,
    pub body: String,
    pub content_hash: String,
    pub updated_at: DateTime<Utc>,
}

#[apply(seaorm_relation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
