use az_derive_aliases::{apply, seaorm_entity_model_eq, seaorm_relation};
use sea_orm::entity::prelude::*;
use uuid::Uuid;

#[apply(seaorm_entity_model_eq)]
#[sea_orm(table_name = "admin_software_install_methods")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub software_id: Uuid,
    pub platform: String,
    pub installer_kind: String,
    pub label: String,
    pub package_id: String,
    pub asset_item_id: Option<String>,
    pub command_text: String,
    pub note: String,
    pub priority: i32,
}

#[apply(seaorm_relation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
