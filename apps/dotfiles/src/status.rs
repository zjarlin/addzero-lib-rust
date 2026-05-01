use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{DotfilesError, Result, io_error};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaskInfo {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct StatusStore {
    path: PathBuf,
}

impl StatusStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn ensure_dir(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| io_error(parent, err))?;
        }
        Ok(())
    }

    pub fn print(&self) -> Result<()> {
        let tasks = self.load()?;
        if tasks.is_empty() {
            println!("没有任务正在执行");
            return Ok(());
        }

        for task in tasks.values() {
            let status = match task.status {
                Status::Pending => "待执行",
                Status::Running => "执行中",
                Status::Completed => "已完成",
                Status::Failed => "失败",
            };
            println!("{} - {}", task.description, status);
        }
        Ok(())
    }

    pub fn record_completed(&self, name: &str, description: &str) -> Result<()> {
        let mut tasks = self.load()?;
        let now = now_millis();
        let task = TaskInfo {
            id: format!("{now}-{name}"),
            name: name.to_string(),
            description: description.to_string(),
            status: Status::Completed,
            error_message: None,
            start_time: Some(now),
            end_time: Some(now),
        };
        tasks.insert(task.id.clone(), task);
        self.save(&tasks)
    }

    fn load(&self) -> Result<BTreeMap<String, TaskInfo>> {
        if !self.path.exists() {
            return Ok(BTreeMap::new());
        }
        let content =
            std::fs::read_to_string(&self.path).map_err(|err| io_error(&self.path, err))?;
        if content.trim().is_empty() {
            return Ok(BTreeMap::new());
        }
        serde_json::from_str(&content).map_err(|err| DotfilesError::Json {
            path: self.path.clone(),
            source: err,
        })
    }

    fn save(&self, tasks: &BTreeMap<String, TaskInfo>) -> Result<()> {
        self.ensure_dir()?;
        let content = serde_json::to_string_pretty(tasks)?;
        std::fs::write(&self.path, format!("{content}\n")).map_err(|err| io_error(&self.path, err))
    }
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}
