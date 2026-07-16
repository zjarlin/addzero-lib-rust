use az_aio_platform::core::db::ToastyModelContribution;
use serde::{Deserialize, Serialize};

pub const TABLE_NAME_PREFIX: &str = "biz_drive_center_";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
#[table = "biz_drive_center_drive_tasks"]
pub struct DriveTask {
    #[key]
    pub id: String,
    #[index]
    pub drive_path: String,
    pub action: String,
    pub status: String,
    pub updated_at: String,
}

#[rudi::Singleton(name = "drive-center-toasty-models")]
pub fn drive_center_model_contribution() -> ToastyModelContribution {
    ToastyModelContribution::new(toasty::models!(DriveTask))
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DriveTaskSummary {
    pub id: String,
    pub path: String,
    pub action: String,
    pub status: String,
}

impl From<DriveTask> for DriveTaskSummary {
    fn from(task: DriveTask) -> Self {
        Self {
            id: task.id,
            path: task.drive_path,
            action: task.action,
            status: task.status,
        }
    }
}
