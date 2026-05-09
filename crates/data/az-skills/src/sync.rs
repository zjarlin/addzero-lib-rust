use std::collections::BTreeMap;

use anyhow::Result;
use chrono::Utc;

use crate::fs_repo::FsRepo;
use crate::pg_repo::PgRepo;
use crate::types::{Skill, SkillUpsert, SyncReport};

/// Reconcile both repositories. Strategy: union by name; equal hashes are no-ops;
/// if only one side has a record, copy across; if both differ, the newer
/// `updated_at` wins (with the loser's name recorded as a conflict for the UI).
pub async fn sync_all(pg: &PgRepo, fs: &FsRepo) -> Result<SyncReport> {
    let mut report = SyncReport::default();

    let pg_skills = pg.list().await?;
    let fs_skills = fs.list().await?;

    let mut by_name: BTreeMap<String, (Option<Skill>, Option<Skill>)> = BTreeMap::new();
    for s in pg_skills {
        let name = s.name.clone();
        by_name.entry(name).or_default().0 = Some(s);
    }
    for s in fs_skills {
        let name = s.name.clone();
        by_name.entry(name).or_default().1 = Some(s);
    }

    for (name, pair) in by_name {
        match pair {
            (Some(pg_skill), None) => {
                let upsert = skill_to_upsert(&pg_skill);
                fs.upsert(&upsert).await?;
                report.added_to_fs.push(name);
            }
            (None, Some(fs_skill)) => {
                let upsert = skill_to_upsert(&fs_skill);
                pg.upsert(&upsert, fs_skill.updated_at, &fs_skill.content_hash)
                    .await?;
                report.added_to_pg.push(name);
            }
            (Some(pg_skill), Some(fs_skill)) => {
                if pg_skill.content_hash == fs_skill.content_hash {
                    continue;
                }
                if pg_skill.updated_at >= fs_skill.updated_at {
                    let upsert = skill_to_upsert(&pg_skill);
                    fs.upsert(&upsert).await?;
                    report.updated_in_fs.push(name.clone());
                } else {
                    let upsert = skill_to_upsert(&fs_skill);
                    pg.upsert(&upsert, fs_skill.updated_at, &fs_skill.content_hash)
                        .await?;
                    report.updated_in_pg.push(name.clone());
                }
                report.conflicts.push(name);
            }
            (None, None) => {}
        }
    }

    report.finished_at = Some(Utc::now());
    Ok(report)
}

fn skill_to_upsert(skill: &Skill) -> SkillUpsert {
    SkillUpsert {
        name: skill.name.clone(),
        keywords: skill.keywords.clone(),
        description: skill.description.clone(),
        body: skill.body.clone(),
    }
}
