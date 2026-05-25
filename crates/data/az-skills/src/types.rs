use az_derive_aliases::{apply, serde_code_enum, serde_eq, serde_eq_default};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Where a particular skill record is currently observed to live.
#[apply(serde_code_enum)]
pub enum SkillSource {
    Postgres,
    FileSystem,
    Both,
}

impl SkillSource {
    pub fn merge(self, other: SkillSource) -> SkillSource {
        match (self, other) {
            (SkillSource::Both, _) | (_, SkillSource::Both) => SkillSource::Both,
            (SkillSource::Postgres, SkillSource::FileSystem)
            | (SkillSource::FileSystem, SkillSource::Postgres) => SkillSource::Both,
            (a, _) => a,
        }
    }
}

/// A skill as it lives in our domain (independent of any single backend).
#[apply(serde_eq)]
pub struct Skill {
    pub id: Uuid,
    pub name: String,
    pub keywords: Vec<String>,
    pub description: String,
    pub body: String,
    pub content_hash: String,
    pub updated_at: DateTime<Utc>,
    pub source: SkillSource,
}

/// Payload for create/update operations.
#[apply(serde_eq)]
pub struct SkillUpsert {
    pub name: String,
    pub keywords: Vec<String>,
    pub description: String,
    pub body: String,
}

impl SkillUpsert {
    /// Compute a stable content hash. Sorted keywords + raw description/body.
    pub fn compute_hash(&self) -> String {
        let mut keywords = self.keywords.clone();
        keywords.sort();
        let mut hasher = Sha256::new();
        hasher.update(self.name.as_bytes());
        hasher.update(b"\x00");
        hasher.update(keywords.join(",").as_bytes());
        hasher.update(b"\x00");
        hasher.update(self.description.as_bytes());
        hasher.update(b"\x00");
        hasher.update(self.body.as_bytes());
        let digest = hasher.finalize();
        format!("{:x}", digest)
    }
}

/// Outcome of a sync_all run.
#[apply(serde_eq_default)]
pub struct SyncReport {
    /// PG-only side. Skills that existed in PG but not in fs and were copied
    /// to fs.
    pub added_to_fs: Vec<String>,
    /// fs-only side. Skills that existed in fs but not in PG and were copied
    /// to PG.
    pub added_to_pg: Vec<String>,
    /// Skills updated in fs because PG had a newer version.
    pub updated_in_fs: Vec<String>,
    /// Skills updated in PG because fs had a newer version.
    pub updated_in_pg: Vec<String>,
    /// Skills where both sides had diverged (kept the newer one, but recorded
    /// here so the UI can warn the operator).
    pub conflicts: Vec<String>,
    pub finished_at: Option<DateTime<Utc>>,
}

impl SyncReport {
    pub fn total_changes(&self) -> usize {
        self.added_to_fs.len()
            + self.added_to_pg.len()
            + self.updated_in_fs.len()
            + self.updated_in_pg.len()
    }
}

/// Helper to build a freshly-stamped Skill from an upsert payload.
pub fn skill_from_upsert(input: SkillUpsert, source: SkillSource) -> Skill {
    let content_hash = input.compute_hash();
    Skill {
        id: Uuid::new_v4(),
        name: input.name,
        keywords: input.keywords,
        description: input.description,
        body: input.body,
        content_hash,
        updated_at: Utc::now(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::SkillSource;

    #[test]
    fn skill_source_codes_follow_wire_values() {
        assert_eq!(SkillSource::Postgres.code(), "postgres");
        assert_eq!(SkillSource::FileSystem.code(), "file_system");
        assert_eq!(SkillSource::from_code("both"), Some(SkillSource::Both));
    }
}
