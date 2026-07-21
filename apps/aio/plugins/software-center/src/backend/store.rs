use anyhow::bail;
use az_aio_platform::core::db;
use az_str::transformation::normalized_id_or_else;
use rudi::{Context, DynProvider, Module, modules, providers, singleton};
use std::sync::Arc;
use toasty::stmt::{List, Query};

use crate::backend::model::{SoftwarePackageRecord, SoftwarePackageSummary, TABLE_NAME_PREFIX};

#[derive(Clone)]
pub struct SoftwareCenterStore {
    db: db::Db,
}

impl SoftwareCenterStore {
    pub fn from_shared(db: db::Db) -> Self {
        Self { db }
    }

    pub async fn list_packages(&self) -> anyhow::Result<Vec<SoftwarePackageSummary>> {
        let mut db = self.db.lock().await;
        let records = Query::<List<SoftwarePackageRecord>>::all().exec(&mut *db).await?;
        Ok(records.into_iter().map(Into::into).collect())
    }

    pub async fn upsert_package(
        &self,
        input: SoftwarePackageInput,
    ) -> anyhow::Result<SoftwarePackageSummary> {
        validate_software_package_input(&input)?;
        let id = normalized_id_or_else(input.id, db::new_uuid_id);
        let now = db::timestamp_secs();
        let mut db = self.db.lock().await;
        let existing =
            Query::<List<SoftwarePackageRecord>>::filter(SoftwarePackageRecord::fields().id().eq(&id))
                .first()
                .exec(&mut *db)
                .await?;
        let record = match existing {
            Some(_) => {
                SoftwarePackageRecord::filter(SoftwarePackageRecord::fields().id().eq(&id))
                    .update()
                    .name(input.name)
                    .source_path(input.source_path)
                    .platform(input.platform)
                    .arch(input.arch)
                    .status(input.status.unwrap_or_else(|| "pending".to_string()))
                    .updated_at(now)
                    .exec(&mut *db)
                    .await?;
                Query::<List<SoftwarePackageRecord>>::filter(
                    SoftwarePackageRecord::fields().id().eq(&id),
                )
                .one()
                .exec(&mut *db)
                .await?
            }
            None => {
                SoftwarePackageRecord::create()
                    .id(id)
                    .name(input.name)
                    .source_path(input.source_path)
                    .platform(input.platform)
                    .arch(input.arch)
                    .status(input.status.unwrap_or_else(|| "pending".to_string()))
                    .updated_at(now)
                    .exec(&mut *db)
                    .await?
            }
        };
        Ok(record.into())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SoftwarePackageInput {
    pub id: Option<String>,
    pub name: String,
    pub source_path: String,
    pub platform: String,
    pub arch: String,
    pub status: Option<String>,
}

pub trait SoftwareCenterService: Send + Sync {
    fn plugin_id(&self) -> &'static str;
    fn table_prefix(&self) -> &'static str;
}

#[derive(Clone)]
pub struct SoftwareCenterServiceImpl;

impl SoftwareCenterService for SoftwareCenterServiceImpl {
    fn plugin_id(&self) -> &'static str {
        "software-center"
    }

    fn table_prefix(&self) -> &'static str {
        TABLE_NAME_PREFIX
    }
}

pub struct SoftwareCenterModule;

impl Module for SoftwareCenterModule {
    fn providers() -> Vec<DynProvider> {
        providers![
            singleton(|_| Arc::new(SoftwareCenterServiceImpl) as Arc<dyn SoftwareCenterService>),
            singleton(|cx| SoftwareCenterStore::from_shared(cx.resolve::<db::Db>())),
        ]
    }
}

pub fn build_software_center_context() -> Context {
    Context::create(modules![SoftwareCenterModule])
}

pub fn build_software_center_context_with_db(shared_db: db::Db) -> Context {
    Context::options()
        .singleton(shared_db)
        .create(modules![SoftwareCenterModule])
}

pub fn validate_software_package_input(
    input: &SoftwarePackageInput,
) -> anyhow::Result<()> {
    if input.name.trim().is_empty() {
        bail!("software package name must not be blank");
    }
    if input.source_path.trim().is_empty() {
        bail!("software package source path must not be blank");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_blank_software_package_input() {
        let input = SoftwarePackageInput {
            id: None,
            name: "".to_string(),
            source_path: "/tmp/pkg.dmg".to_string(),
            platform: "macOS".to_string(),
            arch: "arm64".to_string(),
            status: None,
        };
        let error = validate_software_package_input(&input).unwrap_err();
        assert_eq!(error.to_string(), "software package name must not be blank");
    }

    #[test]
    fn rudi_context_resolves_service() {
        let mut context = build_software_center_context();
        let service = context.resolve::<Arc<dyn SoftwareCenterService>>();
        assert_eq!(service.plugin_id(), "software-center");
        assert_eq!(service.table_prefix(), TABLE_NAME_PREFIX);
    }
}
