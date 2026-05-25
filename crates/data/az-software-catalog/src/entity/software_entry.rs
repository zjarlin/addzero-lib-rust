use az_derive_aliases::{apply, seaorm_entity_model_eq, seaorm_relation};
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[apply(seaorm_entity_model_eq)]
#[sea_orm(table_name = "admin_software_entries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub vendor: String,
    pub summary: String,
    pub homepage_url: String,
    pub icon_url: String,
    pub tags: Json,
    pub trial_platforms: Json,
    pub raw: Json,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[apply(seaorm_relation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
