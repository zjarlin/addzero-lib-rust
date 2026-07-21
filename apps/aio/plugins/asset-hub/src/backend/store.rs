use anyhow::bail;
use az_aio_platform::core::db;
use az_str::transformation::normalized_id_or_else;
use rudi::{Context, DynProvider, Module, modules, providers, singleton};
use std::sync::Arc;
use toasty::stmt::{List, Query};

use crate::backend::model::{AssetRecord, AssetSummary, TABLE_NAME_PREFIX};

#[derive(Clone)]
pub struct AssetHubStore {
    db: db::Db,
}

impl AssetHubStore {
    pub fn from_shared(db: db::Db) -> Self {
        Self { db }
    }

    pub async fn list_assets(&self) -> anyhow::Result<Vec<AssetSummary>> {
        let mut db = self.db.lock().await;
        let records = Query::<List<AssetRecord>>::all().exec(&mut *db).await?;
        Ok(records.into_iter().map(Into::into).collect())
    }

    pub async fn upsert_asset(&self, input: AssetInput) -> anyhow::Result<AssetSummary> {
        validate_asset_input(&input)?;
        let id = normalized_id_or_else(input.id, db::new_uuid_id);
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

pub trait AssetHubService: Send + Sync {
    fn plugin_id(&self) -> &'static str;
    fn table_prefix(&self) -> &'static str;
}

#[derive(Clone)]
pub struct AssetHubServiceImpl;

impl AssetHubService for AssetHubServiceImpl {
    fn plugin_id(&self) -> &'static str {
        "asset-hub"
    }

    fn table_prefix(&self) -> &'static str {
        TABLE_NAME_PREFIX
    }
}

pub struct AssetHubModule;

impl Module for AssetHubModule {
    fn providers() -> Vec<DynProvider> {
        providers![
            singleton(|_| Arc::new(AssetHubServiceImpl) as Arc<dyn AssetHubService>),
            singleton(|cx| AssetHubStore::from_shared(cx.resolve::<db::Db>())),
        ]
    }
}

pub fn build_asset_hub_context() -> Context {
    Context::create(modules![AssetHubModule])
}

pub fn build_asset_hub_context_with_db(shared_db: db::Db) -> Context {
    Context::options()
        .singleton(shared_db)
        .create(modules![AssetHubModule])
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

#[cfg(test)]
mod tests {
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
    fn rudi_context_resolves_service() {
        let mut context = build_asset_hub_context();
        let service = context.resolve::<Arc<dyn AssetHubService>>();
        assert_eq!(service.plugin_id(), "asset-hub");
        assert_eq!(service.table_prefix(), TABLE_NAME_PREFIX);
    }
}
