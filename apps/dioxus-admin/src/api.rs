//! Server functions exposed to the dioxus front-end. The DTOs are plain
//! serde structs so they compile on both the client (web/desktop) and the
//! server bundle. Function bodies only run server-side; on the client the
//! `#[server]` macro replaces them with an HTTP call.
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

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

#[cfg(feature = "server")]
mod server_impl {
    use super::*;
    use addzero_skills::{Skill, SkillSource, SkillUpsert, SyncReport};

    impl From<Skill> for SkillDto {
        fn from(s: Skill) -> Self {
            SkillDto {
                name: s.name,
                keywords: s.keywords,
                description: s.description,
                body: s.body,
                content_hash: s.content_hash,
                updated_at: s.updated_at,
                source: match s.source {
                    SkillSource::Postgres => SkillSourceDto::Postgres,
                    SkillSource::FileSystem => SkillSourceDto::FileSystem,
                    SkillSource::Both => SkillSourceDto::Both,
                },
            }
        }
    }

    impl From<SkillUpsertDto> for SkillUpsert {
        fn from(d: SkillUpsertDto) -> Self {
            SkillUpsert {
                name: d.name,
                keywords: d.keywords,
                description: d.description,
                body: d.body,
            }
        }
    }

    pub fn report_to_dto(report: SyncReport, pg_online: bool, fs_root: String) -> SyncReportDto {
        SyncReportDto {
            added_to_fs: report.added_to_fs,
            added_to_pg: report.added_to_pg,
            updated_in_fs: report.updated_in_fs,
            updated_in_pg: report.updated_in_pg,
            conflicts: report.conflicts,
            finished_at: report.finished_at,
            pg_online,
            fs_root,
        }
    }
}

#[server]
pub async fn list_skills() -> Result<Vec<SkillDto>, ServerFnError> {
    let svc = crate::server::service().await;
    let skills = svc
        .list()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(skills.into_iter().map(SkillDto::from).collect())
}

#[server]
pub async fn get_skill(name: String) -> Result<Option<SkillDto>, ServerFnError> {
    let svc = crate::server::service().await;
    let skill = svc
        .get(&name)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(skill.map(SkillDto::from))
}

#[server]
pub async fn upsert_skill(input: SkillUpsertDto) -> Result<SkillDto, ServerFnError> {
    let svc = crate::server::service().await;
    let skill = svc
        .upsert(input.into())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(skill.into())
}

#[server]
pub async fn delete_skill(name: String) -> Result<(), ServerFnError> {
    let svc = crate::server::service().await;
    svc.delete(&name)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[server]
pub async fn sync_skills() -> Result<SyncReportDto, ServerFnError> {
    let svc = crate::server::service().await;
    let pg_online = svc.is_pg_online();
    let fs_root = svc.fs_root_display();
    let report = svc
        .sync_now()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(server_impl::report_to_dto(report, pg_online, fs_root))
}

#[server]
pub async fn server_status() -> Result<SyncReportDto, ServerFnError> {
    let svc = crate::server::service().await;
    let pg_online = svc.is_pg_online();
    let fs_root = svc.fs_root_display();
    let report = svc.last_report().await.unwrap_or_default();
    Ok(server_impl::report_to_dto(report, pg_online, fs_root))
}
