//! Skill management against `~/.agents/skills/<name>/SKILL.md` and an optional
//! Postgres mirror. The service is the single entry point used by the
//! dioxus-admin server functions.
pub mod fs_repo;
pub mod pg_repo;
pub mod sync;
pub mod types;

use anyhow::Result;
use chrono::Utc;
use tokio::sync::Mutex;

pub use fs_repo::FsRepo;
pub use pg_repo::PgRepo;
pub use sync::sync_all;
pub use types::{Skill, SkillSource, SkillUpsert, SyncReport, skill_from_upsert};

/// Top-level facade. Wraps an optional PG repo + a mandatory fs repo and
/// exposes CRUD operations plus a manual `sync_now()`.
pub struct SkillService {
    pg: Option<PgRepo>,
    fs: FsRepo,
    last_report: Mutex<Option<SyncReport>>,
}

impl SkillService {
    pub fn fs_only(fs: FsRepo) -> Self {
        Self {
            pg: None,
            fs,
            last_report: Mutex::new(None),
        }
    }

    pub fn with_pg(fs: FsRepo, pg: PgRepo) -> Self {
        Self {
            pg: Some(pg),
            fs,
            last_report: Mutex::new(None),
        }
    }

    /// Try to attach to PG; on failure, return an fs-only service so the admin
    /// can still operate offline.
    pub async fn try_attach(database_url: Option<&str>, fs: FsRepo) -> Self {
        let Some(url) = database_url.filter(|u| !u.is_empty()) else {
            return Self::fs_only(fs);
        };
        match PgRepo::connect(url).await {
            Ok(pg) => match pg.ensure_schema().await {
                Ok(()) => Self::with_pg(fs, pg),
                Err(err) => {
                    log::warn!("PG schema bootstrap failed, falling back to fs-only: {err:?}");
                    Self::fs_only(fs)
                }
            },
            Err(err) => {
                log::warn!("PG connect failed, falling back to fs-only: {err:?}");
                Self::fs_only(fs)
            }
        }
    }

    pub fn is_pg_online(&self) -> bool {
        self.pg.is_some()
    }

    pub fn fs_root_display(&self) -> String {
        self.fs.root().display().to_string()
    }

    pub async fn list(&self) -> Result<Vec<Skill>> {
        let mut fs_skills = self.fs.list().await?;
        let Some(pg) = &self.pg else {
            return Ok(fs_skills);
        };
        let pg_skills = pg.list().await.unwrap_or_default();
        // Merge by name. Prefer PG row data (source-of-truth when online) but
        // upgrade `source` to Both when both repos hold the record.
        use std::collections::BTreeMap;
        let mut by_name: BTreeMap<String, Skill> = BTreeMap::new();
        for s in fs_skills.drain(..) {
            by_name.insert(s.name.clone(), s);
        }
        for mut s in pg_skills {
            if let Some(prev) = by_name.get(&s.name) {
                if prev.content_hash == s.content_hash {
                    s.source = SkillSource::Both;
                }
            }
            by_name.insert(s.name.clone(), s);
        }
        Ok(by_name.into_values().collect())
    }

    pub async fn get(&self, name: &str) -> Result<Option<Skill>> {
        if let Some(pg) = &self.pg {
            if let Some(skill) = pg.get(name).await? {
                return Ok(Some(skill));
            }
        }
        self.fs.get(name).await
    }

    pub async fn upsert(&self, input: SkillUpsert) -> Result<Skill> {
        let content_hash = input.compute_hash();
        let updated_at = Utc::now();
        let mut fs_result = self.fs.upsert(&input).await?;
        fs_result.content_hash = content_hash.clone();
        fs_result.updated_at = updated_at;

        if let Some(pg) = &self.pg {
            match pg.upsert(&input, updated_at, &content_hash).await {
                Ok(pg_skill) => {
                    return Ok(Skill {
                        source: SkillSource::Both,
                        ..pg_skill
                    });
                }
                Err(err) => {
                    log::warn!("PG upsert failed for {}: {err:?}", input.name);
                }
            }
        }
        Ok(fs_result)
    }

    pub async fn delete(&self, name: &str) -> Result<()> {
        self.fs.delete(name).await?;
        if let Some(pg) = &self.pg {
            if let Err(err) = pg.delete(name).await {
                log::warn!("PG delete failed for {name}: {err:?}");
            }
        }
        Ok(())
    }

    /// Run a full reconcile, store the report on the service, and return it.
    pub async fn sync_now(&self) -> Result<SyncReport> {
        let report = if let Some(pg) = &self.pg {
            sync_all(pg, &self.fs).await?
        } else {
            SyncReport {
                finished_at: Some(Utc::now()),
                ..Default::default()
            }
        };
        let mut slot = self.last_report.lock().await;
        *slot = Some(report.clone());
        Ok(report)
    }

    pub async fn last_report(&self) -> Option<SyncReport> {
        self.last_report.lock().await.clone()
    }
}
