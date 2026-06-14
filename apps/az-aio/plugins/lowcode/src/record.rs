//! Runtime record store — generic key-value records for generated AppScreens.
//!
//! Maps `model_id → Vec<Record>` where each Record is `HashMap<String, String>`.
//! This is the "data layer" that low-code generated pages CRUD against.

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single record row — all values are strings for simplicity.
pub type Record = HashMap<String, String>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordWithId {
    pub id: String,
    #[serde(flatten)]
    pub fields: Record,
}

#[derive(Default)]
struct RecordMemStore {
    /// model_id → list of records
    records: Mutex<HashMap<String, Vec<RecordWithId>>>,
}

/// Thread-safe runtime record store.
#[derive(Clone, Default)]
pub struct RecordStore {
    mem: Arc<RecordMemStore>,
}

static GLOBAL_RECORDS: std::sync::OnceLock<RecordStore> = std::sync::OnceLock::new();

impl RecordStore {
    pub fn global() -> RecordStore {
        GLOBAL_RECORDS.get_or_init(RecordStore::default).clone()
    }

    pub fn list(&self, model_id: &str) -> Vec<RecordWithId> {
        let records = self.mem.records.lock();
        records.get(model_id).cloned().unwrap_or_default()
    }

    pub fn get(&self, model_id: &str, record_id: &str) -> Option<RecordWithId> {
        let records = self.mem.records.lock();
        records
            .get(model_id)
            .and_then(|list| list.iter().find(|r| r.id == record_id).cloned())
    }

    pub fn create(&self, model_id: &str, fields: Record) -> RecordWithId {
        let id = Uuid::new_v4().to_string();
        let record = RecordWithId { id, fields };
        let mut records = self.mem.records.lock();
        records
            .entry(model_id.to_string())
            .or_default()
            .push(record.clone());
        record
    }

    pub fn update(&self, model_id: &str, record_id: &str, fields: Record) -> Option<RecordWithId> {
        let mut records = self.mem.records.lock();
        let list = records.get_mut(model_id)?;
        let entry = list.iter_mut().find(|r| r.id == record_id)?;
        entry.fields = fields;
        Some(entry.clone())
    }

    pub fn delete(&self, model_id: &str, record_id: &str) -> bool {
        let mut records = self.mem.records.lock();
        if let Some(list) = records.get_mut(model_id) {
            let len_before = list.len();
            list.retain(|r| r.id != record_id);
            return list.len() < len_before;
        }
        false
    }

    /// Remove all records for a given model (cascade on model delete).
    pub fn delete_model_records(&self, model_id: &str) {
        self.mem.records.lock().remove(model_id);
    }

    /// Seed demo records for the built-in models.
    pub fn seed_demo(&self) {
        let mut records = self.mem.records.lock();
        if !records.is_empty() {
            return;
        }

        // Project records
        records.insert(
            "demo-proj-001".into(),
            vec![
                RecordWithId {
                    id: "rec-proj-1".into(),
                    fields: HashMap::from([
                        ("name".into(), "项目 Alpha".into()),
                        ("status".into(), "活跃".into()),
                        ("start_date".into(), "2026-01-15".into()),
                        ("budget".into(), "500000".into()),
                    ]),
                },
                RecordWithId {
                    id: "rec-proj-2".into(),
                    fields: HashMap::from([
                        ("name".into(), "项目 Beta".into()),
                        ("status".into(), "草稿".into()),
                        ("start_date".into(), "2026-03-01".into()),
                        ("budget".into(), "120000".into()),
                    ]),
                },
                RecordWithId {
                    id: "rec-proj-3".into(),
                    fields: HashMap::from([
                        ("name".into(), "项目 Gamma".into()),
                        ("status".into(), "已完成".into()),
                        ("start_date".into(), "2025-11-20".into()),
                        ("budget".into(), "800000".into()),
                    ]),
                },
            ],
        );

        // Employee records
        records.insert(
            "demo-emp-001".into(),
            vec![
                RecordWithId {
                    id: "rec-emp-1".into(),
                    fields: HashMap::from([
                        ("name".into(), "张三".into()),
                        ("email".into(), "zhangsan@example.com".into()),
                        ("project_id".into(), "rec-proj-1".into()),
                        ("manager_id".into(), "rec-emp-2".into()),
                    ]),
                },
                RecordWithId {
                    id: "rec-emp-2".into(),
                    fields: HashMap::from([
                        ("name".into(), "李四".into()),
                        ("email".into(), "lisi@example.com".into()),
                        ("project_id".into(), "rec-proj-1".into()),
                        ("manager_id".into(), "".into()),
                    ]),
                },
            ],
        );

        // Organization records
        records.insert(
            "demo-org-001".into(),
            vec![
                RecordWithId {
                    id: "rec-org-1".into(),
                    fields: HashMap::from([
                        ("name".into(), "总公司".into()),
                        ("parent_id".into(), "".into()),
                        ("level".into(), "1".into()),
                    ]),
                },
                RecordWithId {
                    id: "rec-org-2".into(),
                    fields: HashMap::from([
                        ("name".into(), "研发部".into()),
                        ("parent_id".into(), "rec-org-1".into()),
                        ("level".into(), "2".into()),
                    ]),
                },
                RecordWithId {
                    id: "rec-org-3".into(),
                    fields: HashMap::from([
                        ("name".into(), "市场部".into()),
                        ("parent_id".into(), "rec-org-1".into()),
                        ("level".into(), "2".into()),
                    ]),
                },
                RecordWithId {
                    id: "rec-org-4".into(),
                    fields: HashMap::from([
                        ("name".into(), "前端组".into()),
                        ("parent_id".into(), "rec-org-2".into()),
                        ("level".into(), "3".into()),
                    ]),
                },
                RecordWithId {
                    id: "rec-org-5".into(),
                    fields: HashMap::from([
                        ("name".into(), "后端组".into()),
                        ("parent_id".into(), "rec-org-2".into()),
                        ("level".into(), "3".into()),
                    ]),
                },
            ],
        );
    }
}
