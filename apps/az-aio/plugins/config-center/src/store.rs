use std::sync::Arc;

use shaku::{Component, Interface, module};
use toasty::stmt::{List, Query};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    error::{ConfigCenterError, ConfigCenterResult},
    model::{ConfigEntry, ConfigEntrySummary, TABLE_NAME_PREFIX},
};

#[derive(Clone)]
pub struct ConfigCenterStore {
    db: Arc<Mutex<toasty::Db>>,
}

impl ConfigCenterStore {
    pub async fn connect(database_url: &str) -> ConfigCenterResult<Self> {
        let database_url = validate_database_url(Some(database_url))?;
        let db = toasty::Db::builder()
            .models(toasty::models!(ConfigEntry))
            .table_name_prefix(TABLE_NAME_PREFIX)
            .connect(database_url)
            .await?;
        db.push_schema().await?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }

    pub async fn list_entries(
        &self,
        namespace: &str,
    ) -> ConfigCenterResult<Vec<ConfigEntrySummary>> {
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
    ) -> ConfigCenterResult<ConfigEntrySummary> {
        validate_config_entry_input(&input)?;
        let id = normalized_id(input.id);
        let now = timestamp_string();
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

pub trait ConfigCenterService: Interface {
    fn plugin_id(&self) -> &'static str;
    fn table_prefix(&self) -> &'static str;
}

#[derive(Component)]
#[shaku(interface = ConfigCenterService)]
pub struct ConfigCenterServiceImpl;

impl ConfigCenterService for ConfigCenterServiceImpl {
    fn plugin_id(&self) -> &'static str {
        "config-center"
    }

    fn table_prefix(&self) -> &'static str {
        TABLE_NAME_PREFIX
    }
}

module! {
    pub ConfigCenterModule {
        components = [ConfigCenterServiceImpl],
        providers = []
    }
}

pub fn build_config_center_module() -> ConfigCenterModule {
    ConfigCenterModule::builder().build()
}

pub fn validate_database_url(value: Option<&str>) -> ConfigCenterResult<&str> {
    let value = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(ConfigCenterError::MissingDatabaseUrl)?;
    Ok(value)
}

pub fn validate_config_entry_input(input: &ConfigEntryInput) -> ConfigCenterResult<()> {
    if input.key.trim().is_empty() {
        return Err(ConfigCenterError::BlankKey);
    }
    if input.value.trim().is_empty() {
        return Err(ConfigCenterError::BlankValue);
    }
    Ok(())
}

fn normalize_namespace(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        "az-aio.dev".to_string()
    } else {
        value.to_string()
    }
}

fn normalized_id(value: Option<String>) -> String {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

fn timestamp_string() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

#[cfg(test)]
mod tests {
    use shaku::HasComponent;

    use super::*;

    #[test]
    fn validates_database_url() {
        assert_eq!(
            validate_database_url(Some(" postgresql://localhost/config ")).unwrap(),
            "postgresql://localhost/config"
        );
        assert!(matches!(
            validate_database_url(None),
            Err(ConfigCenterError::MissingDatabaseUrl)
        ));
    }

    #[test]
    fn rejects_blank_config_entry_input() {
        let input = ConfigEntryInput {
            id: None,
            namespace: "az-aio.dev".to_string(),
            key: "".to_string(),
            value: "secret".to_string(),
        };
        assert!(matches!(
            validate_config_entry_input(&input),
            Err(ConfigCenterError::BlankKey)
        ));
    }

    #[test]
    fn shaku_module_resolves_service() {
        let module = build_config_center_module();
        let service: &dyn ConfigCenterService = module.resolve_ref();
        assert_eq!(service.plugin_id(), "config-center");
        assert_eq!(service.table_prefix(), TABLE_NAME_PREFIX);
    }
}
