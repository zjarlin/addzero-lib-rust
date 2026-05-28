use az_derive_aliases::{apply, serde_code_enum, serde_eq, serde_eq_default};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// 某条技能记录当前被观测到的存放位置。
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

/// 领域层中的技能记录，不绑定任一具体后端。
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

/// 创建或更新技能时使用的输入载荷。
#[apply(serde_eq)]
pub struct SkillUpsert {
    pub name: String,
    pub keywords: Vec<String>,
    pub description: String,
    pub body: String,
}

impl SkillUpsert {
    /// 计算稳定的内容哈希；关键词排序后再拼接原始描述和正文。
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

/// 一次 `sync_all` 执行后的同步结果。
#[apply(serde_eq_default)]
pub struct SyncReport {
    /// 仅存在于 PG 的技能，已被复制到文件系统。
    pub added_to_fs: Vec<String>,
    /// 仅存在于文件系统的技能，已被复制到 PG。
    pub added_to_pg: Vec<String>,
    /// 因 PG 版本更新而在文件系统侧被更新的技能。
    pub updated_in_fs: Vec<String>,
    /// 因文件系统版本更新而在 PG 侧被更新的技能。
    pub updated_in_pg: Vec<String>,
    /// 两侧内容发生分叉的技能；同步保留较新的版本，同时记录给 UI 提醒操作者。
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

/// 根据 upsert 载荷构造带当前时间戳的新技能记录。
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
