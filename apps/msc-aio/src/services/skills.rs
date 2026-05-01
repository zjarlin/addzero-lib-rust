use std::rc::Rc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use super::LocalBoxFuture;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum SkillSourceDto {
    Postgres,
    #[default]
    FileSystem,
    Both,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SkillDto {
    pub name: String,
    pub keywords: Vec<String>,
    pub description: String,
    pub body: String,
    pub content_hash: String,
    pub updated_at: DateTime<Utc>,
    pub source: SkillSourceDto,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillUpsertDto {
    pub name: String,
    pub keywords: Vec<String>,
    pub description: String,
    pub body: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncReportDto {
    pub added_to_fs: Vec<String>,
    pub added_to_pg: Vec<String>,
    pub updated_in_fs: Vec<String>,
    pub updated_in_pg: Vec<String>,
    pub conflicts: Vec<String>,
    pub finished_at: Option<DateTime<Utc>>,
    pub pg_online: bool,
    pub fs_root: String,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum SkillServiceError {
    #[error("{0}")]
    Message(String),
}

impl SkillServiceError {
    pub fn new(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

pub type SkillServiceResult<T> = Result<T, SkillServiceError>;

pub trait SkillsApi: 'static {
    fn list_skills(&self) -> LocalBoxFuture<'_, SkillServiceResult<Vec<SkillDto>>>;

    fn get_skill(&self, name: String) -> LocalBoxFuture<'_, SkillServiceResult<Option<SkillDto>>>;

    fn upsert_skill(
        &self,
        input: SkillUpsertDto,
    ) -> LocalBoxFuture<'_, SkillServiceResult<SkillDto>>;

    fn delete_skill(&self, name: String) -> LocalBoxFuture<'_, SkillServiceResult<()>>;

    fn sync_skills(&self) -> LocalBoxFuture<'_, SkillServiceResult<SyncReportDto>>;

    fn server_status(&self) -> LocalBoxFuture<'_, SkillServiceResult<SyncReportDto>>;
}

pub type SharedSkillsApi = Rc<dyn SkillsApi>;

pub fn default_skills_api() -> SharedSkillsApi {
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(BrowserSkillsApi)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(EmbeddedSkillsApi)
    }
}

#[cfg(target_arch = "wasm32")]
struct BrowserSkillsApi;

#[cfg(target_arch = "wasm32")]
impl SkillsApi for BrowserSkillsApi {
    fn list_skills(&self) -> LocalBoxFuture<'_, SkillServiceResult<Vec<SkillDto>>> {
        Box::pin(async move {
            super::browser_http::get_json("/api/skills")
                .await
                .map_err(SkillServiceError::new)
        })
    }

    fn get_skill(&self, name: String) -> LocalBoxFuture<'_, SkillServiceResult<Option<SkillDto>>> {
        Box::pin(async move {
            super::browser_http::get_json(&format!("/api/skills/{name}"))
                .await
                .map_err(SkillServiceError::new)
        })
    }

    fn upsert_skill(
        &self,
        input: SkillUpsertDto,
    ) -> LocalBoxFuture<'_, SkillServiceResult<SkillDto>> {
        Box::pin(async move {
            super::browser_http::post_json("/api/skills/upsert", &input)
                .await
                .map_err(SkillServiceError::new)
        })
    }

    fn delete_skill(&self, name: String) -> LocalBoxFuture<'_, SkillServiceResult<()>> {
        Box::pin(async move {
            super::browser_http::delete_empty(&format!("/api/skills/{name}"))
                .await
                .map_err(SkillServiceError::new)
        })
    }

    fn sync_skills(&self) -> LocalBoxFuture<'_, SkillServiceResult<SyncReportDto>> {
        Box::pin(async move {
            let payload = serde_json::json!({});
            super::browser_http::post_json("/api/skills/sync", &payload)
                .await
                .map_err(SkillServiceError::new)
        })
    }

    fn server_status(&self) -> LocalBoxFuture<'_, SkillServiceResult<SyncReportDto>> {
        Box::pin(async move {
            super::browser_http::get_json("/api/skills/status")
                .await
                .map_err(SkillServiceError::new)
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct EmbeddedSkillsApi;

#[cfg(not(target_arch = "wasm32"))]
impl SkillsApi for EmbeddedSkillsApi {
    fn list_skills(&self) -> LocalBoxFuture<'_, SkillServiceResult<Vec<SkillDto>>> {
        Box::pin(async move {
            let backend = crate::server::services().await;
            backend
                .skills
                .list()
                .await
                .map(|skills| skills.into_iter().map(skill_to_dto).collect())
                .map_err(|err| SkillServiceError::new(err.to_string()))
        })
    }

    fn get_skill(&self, name: String) -> LocalBoxFuture<'_, SkillServiceResult<Option<SkillDto>>> {
        Box::pin(async move {
            let backend = crate::server::services().await;
            backend
                .skills
                .get(name.as_str())
                .await
                .map(|skill| skill.map(skill_to_dto))
                .map_err(|err| SkillServiceError::new(err.to_string()))
        })
    }

    fn upsert_skill(
        &self,
        input: SkillUpsertDto,
    ) -> LocalBoxFuture<'_, SkillServiceResult<SkillDto>> {
        Box::pin(async move {
            let backend = crate::server::services().await;
            backend
                .skills
                .upsert(addzero_skills::SkillUpsert {
                    name: input.name,
                    keywords: input.keywords,
                    description: input.description,
                    body: input.body,
                })
                .await
                .map(skill_to_dto)
                .map_err(|err| SkillServiceError::new(err.to_string()))
        })
    }

    fn delete_skill(&self, name: String) -> LocalBoxFuture<'_, SkillServiceResult<()>> {
        Box::pin(async move {
            let backend = crate::server::services().await;
            backend
                .skills
                .delete(name.as_str())
                .await
                .map_err(|err| SkillServiceError::new(err.to_string()))
        })
    }

    fn sync_skills(&self) -> LocalBoxFuture<'_, SkillServiceResult<SyncReportDto>> {
        Box::pin(async move {
            let backend = crate::server::services().await;
            backend
                .skills
                .sync_now()
                .await
                .map(|report| SyncReportDto {
                    added_to_fs: report.added_to_fs,
                    added_to_pg: report.added_to_pg,
                    updated_in_fs: report.updated_in_fs,
                    updated_in_pg: report.updated_in_pg,
                    conflicts: report.conflicts,
                    finished_at: report.finished_at,
                    pg_online: backend.skills.is_pg_online(),
                    fs_root: backend.skills.fs_root_display(),
                })
                .map_err(|err| SkillServiceError::new(err.to_string()))
        })
    }

    fn server_status(&self) -> LocalBoxFuture<'_, SkillServiceResult<SyncReportDto>> {
        Box::pin(async move {
            let backend = crate::server::services().await;
            let report = backend.skills.last_report().await.unwrap_or_default();
            Ok(SyncReportDto {
                added_to_fs: report.added_to_fs,
                added_to_pg: report.added_to_pg,
                updated_in_fs: report.updated_in_fs,
                updated_in_pg: report.updated_in_pg,
                conflicts: report.conflicts,
                finished_at: report.finished_at,
                pg_online: backend.skills.is_pg_online(),
                fs_root: backend.skills.fs_root_display(),
            })
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn skill_to_dto(skill: addzero_skills::Skill) -> SkillDto {
    SkillDto {
        name: skill.name,
        keywords: skill.keywords,
        description: skill.description,
        body: skill.body,
        content_hash: skill.content_hash,
        updated_at: skill.updated_at,
        source: match skill.source {
            addzero_skills::SkillSource::Postgres => SkillSourceDto::Postgres,
            addzero_skills::SkillSource::FileSystem => SkillSourceDto::FileSystem,
            addzero_skills::SkillSource::Both => SkillSourceDto::Both,
        },
    }
}
