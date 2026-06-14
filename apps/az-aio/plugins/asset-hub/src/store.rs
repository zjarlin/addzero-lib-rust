use anyhow::bail;
use az_aio_shared::db;
use shaku::{Component, Interface, module};
use toasty::stmt::{List, Query};
use uuid::Uuid;

use crate::{
    model::{AssetRecord, AssetSummary, TABLE_NAME_PREFIX},
};

#[derive(Clone)]
pub struct AssetHubStore {
    db: db::SharedDb,
}

impl AssetHubStore {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let database_url = db::verify_database_url(database_url)?;
        let toasty = toasty::Db::builder()
            .models(toasty::models!(AssetRecord))
            .table_name_prefix(TABLE_NAME_PREFIX)
            .connect(database_url)
            .await?;
        toasty.push_schema().await?;
        Ok(Self {
            db: db::SharedDb::new(toasty),
        })
    }

    pub async fn list_assets(&self) -> anyhow::Result<Vec<AssetSummary>> {
        let mut db = self.db.lock().await;
        let records = Query::<List<AssetRecord>>::all().exec(&mut *db).await?;
        Ok(records.into_iter().map(Into::into).collect())
    }

    pub async fn upsert_asset(&self, input: AssetInput) -> anyhow::Result<AssetSummary> {
        validate_asset_input(&input)?;
        let id = normalized_id(input.id);
        let now = db::timestamp_secs();
        let mut db = self.db.lock().await;
        let existing = Query::<List<AssetRecord>>::filter(AssetRecord::fields().id().eq(&id))
            .first()
            .exec(&mut *db)
            .await?;
        let record = match existing {
            Some(_) => {
                AssetRecord::filter(AssetRecord::fields().id().eq(&id))
                    .update()
                    .kind(input.kind)
                    .title(input.title)
                    .status(input.status)
                    .source(input.source)
                    .updated_at(now)
                    .exec(&mut *db)
                    .await?;
                Query::<List<AssetRecord>>::filter(AssetRecord::fields().id().eq(&id))
                    .one()
                    .exec(&mut *db)
                    .await?
            }
            None => {
                AssetRecord::create()
                    .id(id)
                    .kind(input.kind)
                    .title(input.title)
                    .status(input.status)
                    .source(input.source)
                    .updated_at(now)
                    .exec(&mut *db)
                    .await?
            }
        };
        Ok(record.into())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetInput {
    pub id: Option<String>,
    pub kind: String,
    pub title: String,
    pub status: String,
    pub source: String,
}

pub trait AssetHubService: Interface {
    fn plugin_id(&self) -> &'static str;
    fn table_prefix(&self) -> &'static str;
}

#[derive(Component)]
#[shaku(interface = AssetHubService)]
pub struct AssetHubServiceImpl;

impl AssetHubService for AssetHubServiceImpl {
    fn plugin_id(&self) -> &'static str {
        "asset-hub"
    }

    fn table_prefix(&self) -> &'static str {
        TABLE_NAME_PREFIX
    }
}

module! {
    pub AssetHubModule {
        components = [AssetHubServiceImpl],
        providers = []
    }
}

pub fn build_asset_hub_module() -> AssetHubModule {
    AssetHubModule::builder().build()
}

pub fn validate_asset_input(input: &AssetInput) -> anyhow::Result<()> {
    if input.title.trim().is_empty() {
        bail!("asset title must not be blank");
    }
    if input.status.trim().is_empty() {
        bail!("asset status must not be blank");
    }
    Ok(())
}

fn normalized_id(value: Option<String>) -> String {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

#[cfg(test)]
mod tests {
    use shaku::HasComponent;

    use super::*;

    #[test]
    fn rejects_blank_asset_input() {
        let input = AssetInput {
            id: None,
            kind: "skill".to_string(),
            title: " ".to_string(),
            status: "active".to_string(),
            source: "test".to_string(),
        };
        let error = validate_asset_input(&input).unwrap_err();
        assert_eq!(error.to_string(), "asset title must not be blank");
    }

    #[test]
    fn shaku_module_resolves_service() {
        let module = build_asset_hub_module();
        let service: &dyn AssetHubService = module.resolve_ref();
        assert_eq!(service.plugin_id(), "asset-hub");
        assert_eq!(service.table_prefix(), TABLE_NAME_PREFIX);
    }
}
