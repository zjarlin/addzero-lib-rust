use serde::{Deserialize, Serialize};

pub const TABLE_NAME_PREFIX: &str = "biz_drive_center_";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, toasty::Model)]
pub struct DriveTask {
    #[key]
    pub id: String,
    #[index]
    pub path: String,
    pub action: String,
    pub status: String,
    pub updated_at: String,
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
            path: task.path,
            action: task.action,
            status: task.status,
        }
    }
}
