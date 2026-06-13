use std::sync::Arc;

use anyhow::{anyhow, bail};
use shaku::{Component, Interface, module};
use toasty::stmt::{List, Query};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    model::{DriveTask, DriveTaskSummary, TABLE_NAME_PREFIX},
};

#[derive(Clone)]
pub struct DriveCenterStore {
    db: Arc<Mutex<toasty::Db>>,
}

impl DriveCenterStore {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let database_url = validate_database_url(Some(database_url))?;
        let db = toasty::Db::builder()
            .models(toasty::models!(DriveTask))
            .table_name_prefix(TABLE_NAME_PREFIX)
            .connect(database_url)
            .await?;
        db.push_schema().await?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }

    pub async fn list_tasks(&self) -> anyhow::Result<Vec<DriveTaskSummary>> {
        let mut db = self.db.lock().await;
        let tasks = Query::<List<DriveTask>>::all().exec(&mut *db).await?;
        Ok(tasks.into_iter().map(Into::into).collect())
    }

    pub async fn enqueue_task(&self, input: DriveTaskInput) -> anyhow::Result<DriveTaskSummary> {
        validate_drive_task_input(&input)?;
        let now = timestamp_string();
        let mut db = self.db.lock().await;
        let task = DriveTask::create()
            .id(normalized_id(input.id))
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

pub trait DriveCenterService: Interface {
    fn plugin_id(&self) -> &'static str;
    fn table_prefix(&self) -> &'static str;
}

#[derive(Component)]
#[shaku(interface = DriveCenterService)]
pub struct DriveCenterServiceImpl;

impl DriveCenterService for DriveCenterServiceImpl {
    fn plugin_id(&self) -> &'static str {
        "drive-center"
    }

    fn table_prefix(&self) -> &'static str {
        TABLE_NAME_PREFIX
    }
}

module! {
    pub DriveCenterModule {
        components = [DriveCenterServiceImpl],
        providers = []
    }
}

pub fn build_drive_center_module() -> DriveCenterModule {
    DriveCenterModule::builder().build()
}

pub fn validate_database_url(value: Option<&str>) -> anyhow::Result<&str> {
    let value = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("missing drive-center database url"))?;
    Ok(value)
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
            validate_database_url(Some(" postgresql://localhost/drive ")).unwrap(),
            "postgresql://localhost/drive"
        );
        let error = validate_database_url(None).unwrap_err();
        assert_eq!(error.to_string(), "missing drive-center database url");
    }

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
    fn shaku_module_resolves_service() {
        let module = build_drive_center_module();
        let service: &dyn DriveCenterService = module.resolve_ref();
        assert_eq!(service.plugin_id(), "drive-center");
        assert_eq!(service.table_prefix(), TABLE_NAME_PREFIX);
    }
}
