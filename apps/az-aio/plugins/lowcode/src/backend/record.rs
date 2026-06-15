//! Runtime record store — generic key-value records for generated AppScreens.
//!
//! Maps `model_id → Vec<Record>` where each Record is `HashMap<String, String>`.
//! This is the "data layer" that low-code generated pages CRUD against.

use std::collections::HashMap;
use std::sync::Arc;

use az_aio_platform::db::SharedDb;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::backend::model::LowcodeRecord;

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
    records: Mutex<HashMap<String, Vec<RecordWithId>>>,
}

/// Thread-safe runtime record store with dual-path:
/// PostgreSQL via toasty when connected, in-memory otherwise.
#[derive(Clone)]
pub struct RecordStore {
    mem: Arc<RecordMemStore>,
    db: Option<SharedDb>,
}

impl Default for RecordStore {
    fn default() -> Self {
        Self {
            mem: Arc::new(RecordMemStore::default()),
            db: None,
        }
    }
}

static GLOBAL_RECORDS: std::sync::OnceLock<RecordStore> = std::sync::OnceLock::new();

impl RecordStore {
    pub fn global() -> RecordStore {
        GLOBAL_RECORDS.get_or_init(RecordStore::default).clone()
    }

    pub fn init_db(db: SharedDb) {
        let _ = GLOBAL_RECORDS.set(Self::with_db(db));
    }

    pub fn with_db(db: SharedDb) -> Self {
        Self {
            mem: Arc::new(RecordMemStore::default()),
            db: Some(db),
        }
    }

    pub fn in_memory() -> Self {
        Self::default()
    }

    pub fn list(&self, model_id: &str) -> Vec<RecordWithId> {
        if let Some(ref db) = self.db {
            if let Ok(rt) = tokio::runtime::Runtime::new() {
                return rt.block_on(async {
                    let mut db = db.lock().await;
                    use toasty::stmt::{List, Query};
                    let rows = Query::<List<LowcodeRecord>>::filter(
                        LowcodeRecord::fields().model_id().eq(model_id),
                    )
                    .exec(&mut *db)
                    .await
                    .unwrap_or_default();
                    rows.into_iter()
                        .map(|r| RecordWithId {
                            id: r.id,
                            fields: serde_json::from_str(&r.fields_json).unwrap_or_default(),
                        })
                        .collect()
                });
            }
        }
        let records = self.mem.records.lock();
        records.get(model_id).cloned().unwrap_or_default()
    }

    pub fn get(&self, model_id: &str, record_id: &str) -> Option<RecordWithId> {
        if let Some(ref db) = self.db {
            if let Ok(rt) = tokio::runtime::Runtime::new() {
                return rt.block_on(async {
                    let mut db = db.lock().await;
                    use toasty::stmt::{List, Query};
                    Query::<List<LowcodeRecord>>::filter(
                        LowcodeRecord::fields()
                            .model_id()
                            .eq(model_id)
                            .and(LowcodeRecord::fields().id().eq(record_id)),
                    )
                    .first()
                    .exec(&mut *db)
                    .await
                    .ok()
                    .flatten()
                    .map(|r| RecordWithId {
                        id: r.id,
                        fields: serde_json::from_str(&r.fields_json).unwrap_or_default(),
                    })
                });
            }
        }
        let records = self.mem.records.lock();
        records
            .get(model_id)
            .and_then(|list| list.iter().find(|r| r.id == record_id).cloned())
    }

    pub fn create(&self, model_id: &str, fields: Record) -> RecordWithId {
        let id = Uuid::new_v4().to_string();
        let record = RecordWithId {
            id: id.clone(),
            fields: fields.clone(),
        };

        if let Some(ref db) = self.db {
            if let Ok(rt) = tokio::runtime::Runtime::new() {
                let fields_json = serde_json::to_string(&fields).unwrap_or_default();
                let now = chrono::Utc::now().to_rfc3339();
                let _ = rt.block_on(async {
                    let mut db = db.lock().await;
                    LowcodeRecord::create()
                        .id(&id)
                        .model_id(model_id)
                        .fields_json(&fields_json)
                        .created_at(&now)
                        .updated_at(&now)
                        .exec(&mut *db)
                        .await
                });
                return record;
            }
        }

        let mut records = self.mem.records.lock();
        records
            .entry(model_id.to_string())
            .or_default()
            .push(record.clone());
        record
    }

    /// Update a record. The `fields` map is merged into the existing record's fields
    /// rather than replacing them, so partial updates (single-field edits) work correctly.
    pub fn update(&self, model_id: &str, record_id: &str, fields: Record) -> Option<RecordWithId> {
        if let Some(ref db) = self.db {
            if let Ok(rt) = tokio::runtime::Runtime::new() {
                return rt.block_on(async {
                    // Load existing, merge, then persist the full record
                    let existing = {
                        let records = self.mem.records.lock();
                        records
                            .get(model_id)
                            .and_then(|list| list.iter().find(|r| r.id == record_id).cloned())
                    };
                    let full_fields = if let Some(ref rec) = existing {
                        let mut merged = rec.fields.clone();
                        for (k, v) in fields {
                            merged.insert(k, v);
                        }
                        merged
                    } else {
                        fields
                    };
                    let fields_json = serde_json::to_string(&full_fields).unwrap_or_default();
                    let now = chrono::Utc::now().to_rfc3339();
                    let mut db = db.lock().await;
                    LowcodeRecord::filter(LowcodeRecord::fields().id().eq(record_id))
                        .update()
                        .fields_json(&fields_json)
                        .updated_at(&now)
                        .exec(&mut *db)
                        .await
                        .ok();
                    let updated = RecordWithId {
                        id: record_id.to_string(),
                        fields: full_fields.clone(),
                    };
                    // Also update in-memory cache
                    let mut records = self.mem.records.lock();
                    if let Some(list) = records.get_mut(model_id) {
                        if let Some(entry) = list.iter_mut().find(|r| r.id == record_id) {
                            entry.fields = full_fields;
                        }
                    }
                    Some(updated)
                });
            }
        }

        let mut records = self.mem.records.lock();
        let list = records.get_mut(model_id)?;
        let entry = list.iter_mut().find(|r| r.id == record_id)?;
        for (k, v) in fields {
            entry.fields.insert(k, v);
        }
        Some(entry.clone())
    }

    pub fn delete(&self, model_id: &str, record_id: &str) -> bool {
        if let Some(ref db) = self.db {
            if let Ok(rt) = tokio::runtime::Runtime::new() {
                let _ = rt.block_on(async {
                    let mut db = db.lock().await;
                    LowcodeRecord::filter(LowcodeRecord::fields().id().eq(record_id))
                        .delete()
                        .exec(&mut *db)
                        .await
                });
                // Also remove from mem cache
                let mut records = self.mem.records.lock();
                if let Some(list) = records.get_mut(model_id) {
                    let len_before = list.len();
                    list.retain(|r| r.id != record_id);
                    return list.len() < len_before;
                }
                return true;
            }
        }
        let mut records = self.mem.records.lock();
        if let Some(list) = records.get_mut(model_id) {
            let len_before = list.len();
            list.retain(|r| r.id != record_id);
            return list.len() < len_before;
        }
        false
    }

    pub fn delete_model_records(&self, model_id: &str) {
        if let Some(ref db) = self.db {
            if let Ok(rt) = tokio::runtime::Runtime::new() {
                let _ = rt.block_on(async {
                    let mut db = db.lock().await;
                    LowcodeRecord::filter(LowcodeRecord::fields().model_id().eq(model_id))
                        .delete()
                        .exec(&mut *db)
                        .await
                });
            }
        }
        self.mem.records.lock().remove(model_id);
    }

    pub fn seed_demo(&self) {
        let mut records = self.mem.records.lock();
        if !records.is_empty() {
            return;
        }

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
                        ("status".into(), "草案".into()),
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
