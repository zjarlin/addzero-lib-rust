use anyhow::bail;
use az_aio_platform::core::db;
use az_str::transformation::normalized_id_or_else;
use rudi::{Context, DynProvider, Module, modules, providers, singleton};
use std::sync::Arc;
use toasty::stmt::{List, Query};

use crate::backend::model::{DriveTask, DriveTaskSummary, TABLE_NAME_PREFIX};

#[derive(Clone)]
pub struct DriveCenterStore {
    db: db::Db,
}

impl DriveCenterStore {
    pub fn from_shared(db: db::Db) -> Self {
        Self { db }
    }

    pub async fn list_tasks(&self) -> anyhow::Result<Vec<DriveTaskSummary>> {
        let mut db = self.db.lock().await;
        let tasks = Query::<List<DriveTask>>::all().exec(&mut *db).await?;
        Ok(tasks.into_iter().map(Into::into).collect())
    }

    pub async fn enqueue_task(&self, input: DriveTaskInput) -> anyhow::Result<DriveTaskSummary> {
        validate_drive_task_input(&input)?;
        let now = db::timestamp_secs();
        let mut db = self.db.lock().await;
        let task = DriveTask::create()
            .id(normalized_id_or_else(input.id, db::new_uuid_id))
            .drive_path(input.path)
            .action(input.action)
            .status(input.status.unwrap_or_else(|| "queued".to_string()))
            .updated_at(now)
            .exec(&mut *db)
            .await?;
        Ok(task.into())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriveTaskInput {
    pub id: Option<String>,
    pub path: String,
    pub action: String,
    pub status: Option<String>,
}

pub trait DriveCenterService: Send + Sync {
    fn plugin_id(&self) -> &'static str;
    fn table_prefix(&self) -> &'static str;
}

#[derive(Clone)]
pub struct DriveCenterServiceImpl;

impl DriveCenterService for DriveCenterServiceImpl {
    fn plugin_id(&self) -> &'static str {
        "drive-center"
    }

    fn table_prefix(&self) -> &'static str {
        TABLE_NAME_PREFIX
    }
}

pub struct DriveCenterModule;

impl Module for DriveCenterModule {
    fn providers() -> Vec<DynProvider> {
        providers![
            singleton(|_| Arc::new(DriveCenterServiceImpl) as Arc<dyn DriveCenterService>),
            singleton(|cx| DriveCenterStore::from_shared(cx.resolve::<db::Db>())),
        ]
    }
}

pub fn build_drive_center_context() -> Context {
    Context::create(modules![DriveCenterModule])
}

pub fn build_drive_center_context_with_db(shared_db: db::Db) -> Context {
    Context::options()
        .singleton(shared_db)
        .create(modules![DriveCenterModule])
}

pub fn validate_drive_task_input(input: &DriveTaskInput) -> anyhow::Result<()> {
    if input.path.trim().is_empty() {
        bail!("drive path must not be blank");
    }
    if input.action.trim().is_empty() {
        bail!("drive action must not be blank");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_blank_drive_task_input() {
        let input = DriveTaskInput {
            id: None,
            path: "".to_string(),
            action: "sync".to_string(),
            status: None,
        };
        let error = validate_drive_task_input(&input).unwrap_err();
        assert_eq!(error.to_string(), "drive path must not be blank");
    }

    #[test]
    fn rudi_context_resolves_service() {
        let mut context = build_drive_center_context();
        let service = context.resolve::<Arc<dyn DriveCenterService>>();
        assert_eq!(service.plugin_id(), "drive-center");
        assert_eq!(service.table_prefix(), TABLE_NAME_PREFIX);
    }
}
