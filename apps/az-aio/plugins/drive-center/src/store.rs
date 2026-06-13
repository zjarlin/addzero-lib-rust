use std::sync::Arc;

use shaku::{Component, Interface, module};
use toasty::stmt::{List, Query};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    error::{DriveCenterError, DriveCenterResult},
    model::{DriveTask, DriveTaskSummary, TABLE_NAME_PREFIX},
};

#[derive(Clone)]
pub struct DriveCenterStore {
    db: Arc<Mutex<toasty::Db>>,
}

impl DriveCenterStore {
    pub async fn connect(database_url: &str) -> DriveCenterResult<Self> {
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

    pub async fn list_tasks(&self) -> DriveCenterResult<Vec<DriveTaskSummary>> {
        let mut db = self.db.lock().await;
        let tasks = Query::<List<DriveTask>>::all().exec(&mut *db).await?;
        Ok(tasks.into_iter().map(Into::into).collect())
    }

    pub async fn enqueue_task(&self, input: DriveTaskInput) -> DriveCenterResult<DriveTaskSummary> {
        validate_drive_task_input(&input)?;
        let now = timestamp_string();
        let mut db = self.db.lock().await;
        let task = DriveTask::create()
            .id(normalized_id(input.id))
            .path(input.path)
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

pub fn validate_database_url(value: Option<&str>) -> DriveCenterResult<&str> {
    let value = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(DriveCenterError::MissingDatabaseUrl)?;
    Ok(value)
}

pub fn validate_drive_task_input(input: &DriveTaskInput) -> DriveCenterResult<()> {
    if input.path.trim().is_empty() {
        return Err(DriveCenterError::BlankPath);
    }
    if input.action.trim().is_empty() {
        return Err(DriveCenterError::BlankAction);
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
        assert!(matches!(
            validate_database_url(None),
            Err(DriveCenterError::MissingDatabaseUrl)
        ));
    }

    #[test]
    fn rejects_blank_drive_task_input() {
        let input = DriveTaskInput {
            id: None,
            path: "".to_string(),
            action: "sync".to_string(),
            status: None,
        };
        assert!(matches!(
            validate_drive_task_input(&input),
            Err(DriveCenterError::BlankPath)
        ));
    }

    #[test]
    fn shaku_module_resolves_service() {
        let module = build_drive_center_module();
        let service: &dyn DriveCenterService = module.resolve_ref();
        assert_eq!(service.plugin_id(), "drive-center");
        assert_eq!(service.table_prefix(), TABLE_NAME_PREFIX);
    }
}
