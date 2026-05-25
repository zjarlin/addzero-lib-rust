use az_derive_aliases::{apply, seaorm_entity_model, seaorm_relation};
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[apply(seaorm_entity_model)]
#[sea_orm(table_name = "ai_model_providers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub provider: String,
    pub base_url: Option<String>,
    pub default_model: String,
    pub enabled: bool,
    pub key_id: String,
    pub encrypted_api_key: Option<String>,
    pub api_key_configured: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[apply(seaorm_relation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
