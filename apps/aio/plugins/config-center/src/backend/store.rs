use anyhow::bail;
use az_aio_platform::core::db;
use az_str::transformation::normalized_id_or_else;
use rudi::{Context, DynProvider, Module, modules, providers, singleton};
use std::sync::Arc;
use toasty::stmt::{List, Query};

use crate::backend::model::{ConfigEntry, ConfigEntrySummary, TABLE_NAME_PREFIX};

#[derive(Clone)]
pub struct ConfigCenterStore {
    db: db::Db,
}

impl ConfigCenterStore {
    pub fn from_shared(db: db::Db) -> Self {
        Self { db }
    }

    pub async fn list_entries(
        &self,
        namespace: &str,
    ) -> anyhow::Result<Vec<ConfigEntrySummary>> {
        let namespace = normalize_namespace(namespace);
        let mut db = self.db.lock().await;
        let entries =
            Query::<List<ConfigEntry>>::filter(ConfigEntry::fields().namespace().eq(&namespace))
                .exec(&mut *db)
                .await?;
        Ok(entries.into_iter().map(Into::into).collect())
    }

    pub async fn upsert_entry(
        &self,
        input: ConfigEntryInput,
    ) -> anyhow::Result<ConfigEntrySummary> {
        validate_config_entry_input(&input)?;
        let id = normalized_id_or_else(input.id, db::new_uuid_id);
        let now = db::timestamp_secs();
        let mut db = self.db.lock().await;
        let existing = Query::<List<ConfigEntry>>::filter(ConfigEntry::fields().id().eq(&id))
            .first()
            .exec(&mut *db)
            .await?;
        let entry = match existing {
            Some(_) => {
                ConfigEntry::filter(ConfigEntry::fields().id().eq(&id))
                    .update()
                    .namespace(normalize_namespace(&input.namespace))
                    .key(input.key)
                    .value(input.value)
                    .updated_at(now)
                    .exec(&mut *db)
                    .await?;
                Query::<List<ConfigEntry>>::filter(ConfigEntry::fields().id().eq(&id))
                    .one()
                    .exec(&mut *db)
                    .await?
            }
            None => {
                ConfigEntry::create()
                    .id(id)
                    .namespace(normalize_namespace(&input.namespace))
                    .key(input.key)
                    .value(input.value)
                    .updated_at(now)
                    .exec(&mut *db)
                    .await?
            }
        };
        Ok(entry.into())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigEntryInput {
    pub id: Option<String>,
    pub namespace: String,
    pub key: String,
    pub value: String,
}

pub trait ConfigCenterService: Send + Sync {
    fn plugin_id(&self) -> &'static str;
    fn table_prefix(&self) -> &'static str;
}

#[derive(Clone)]
pub struct ConfigCenterServiceImpl;

impl ConfigCenterService for ConfigCenterServiceImpl {
    fn plugin_id(&self) -> &'static str {
        "config-center"
    }

    fn table_prefix(&self) -> &'static str {
        TABLE_NAME_PREFIX
    }
}

pub struct ConfigCenterModule;

impl Module for ConfigCenterModule {
    fn providers() -> Vec<DynProvider> {
        providers![
            singleton(|_| Arc::new(ConfigCenterServiceImpl) as Arc<dyn ConfigCenterService>),
            singleton(|cx| ConfigCenterStore::from_shared(cx.resolve::<db::Db>())),
        ]
    }
}

pub fn build_config_center_context() -> Context {
    Context::create(modules![ConfigCenterModule])
}

pub fn build_config_center_context_with_db(shared_db: db::Db) -> Context {
    Context::options()
        .singleton(shared_db)
        .create(modules![ConfigCenterModule])
}

pub fn validate_config_entry_input(input: &ConfigEntryInput) -> anyhow::Result<()> {
    if input.key.trim().is_empty() {
        bail!("config key must not be blank");
    }
    if input.value.trim().is_empty() {
        bail!("config value must not be blank");
    }
    Ok(())
}

fn normalize_namespace(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        "az-aio".to_string()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_blank_config_entry_input() {
        let input = ConfigEntryInput {
            id: None,
            namespace: "az-aio".to_string(),
            key: "".to_string(),
            value: "secret".to_string(),
        };
        let error = validate_config_entry_input(&input).unwrap_err();
        assert_eq!(error.to_string(), "config key must not be blank");
    }

    #[test]
    fn rudi_context_resolves_service() {
        let mut context = build_config_center_context();
        let service = context.resolve::<Arc<dyn ConfigCenterService>>();
        assert_eq!(service.plugin_id(), "config-center");
        assert_eq!(service.table_prefix(), TABLE_NAME_PREFIX);
    }
}
