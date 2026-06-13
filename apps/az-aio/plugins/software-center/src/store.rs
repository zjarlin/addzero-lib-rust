use std::sync::Arc;

use anyhow::{anyhow, bail};
use shaku::{Component, Interface, module};
use toasty::stmt::{List, Query};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    model::{SoftwarePackageRecord, SoftwarePackageSummary, TABLE_NAME_PREFIX},
};

#[derive(Clone)]
pub struct SoftwareCenterStore {
    db: Arc<Mutex<toasty::Db>>,
}

impl SoftwareCenterStore {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let database_url = validate_database_url(Some(database_url))?;
        let db = toasty::Db::builder()
            .models(toasty::models!(SoftwarePackageRecord))
            .table_name_prefix(TABLE_NAME_PREFIX)
            .connect(database_url)
            .await?;
        db.push_schema().await?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
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
        let id = normalized_id(input.id);
        let now = timestamp_string();
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

pub trait SoftwareCenterService: Interface {
    fn plugin_id(&self) -> &'static str;
    fn table_prefix(&self) -> &'static str;
}

#[derive(Component)]
#[shaku(interface = SoftwareCenterService)]
pub struct SoftwareCenterServiceImpl;

impl SoftwareCenterService for SoftwareCenterServiceImpl {
    fn plugin_id(&self) -> &'static str {
        "software-center"
    }

    fn table_prefix(&self) -> &'static str {
        TABLE_NAME_PREFIX
    }
}

module! {
    pub SoftwareCenterModule {
        components = [SoftwareCenterServiceImpl],
        providers = []
    }
}

pub fn build_software_center_module() -> SoftwareCenterModule {
    SoftwareCenterModule::builder().build()
}

pub fn validate_database_url(value: Option<&str>) -> anyhow::Result<&str> {
    let value = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("missing software-center database url"))?;
    Ok(value)
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
            validate_database_url(Some(" postgresql://localhost/software ")).unwrap(),
            "postgresql://localhost/software"
        );
        let error = validate_database_url(None).unwrap_err();
        assert_eq!(error.to_string(), "missing software-center database url");
    }

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
    fn shaku_module_resolves_service() {
        let module = build_software_center_module();
        let service: &dyn SoftwareCenterService = module.resolve_ref();
        assert_eq!(service.plugin_id(), "software-center");
        assert_eq!(service.table_prefix(), TABLE_NAME_PREFIX);
    }
}
