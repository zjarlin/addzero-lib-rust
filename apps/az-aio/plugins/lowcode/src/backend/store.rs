use std::sync::{Arc, OnceLock};

use az_aio_platform::core::db::SharedDb;
use parking_lot::Mutex;

use crate::backend::model::{AppScreen, LowcodeRecord, MetaField, MetaModel};

automod::dir!(pub "src/backend/store");

static GLOBAL_STORE: OnceLock<LowcodeStore> = OnceLock::new();

#[derive(Clone)]
pub struct LowcodeStore {
    pub(crate) db: Option<SharedDb>,
    pub(crate) mem: Arc<MemStore>,
}

#[derive(Default)]
pub(crate) struct MemStore {
    pub(crate) models: Mutex<Vec<MetaModel>>,
    pub(crate) fields: Mutex<Vec<MetaField>>,
    pub(crate) screens: Mutex<Vec<AppScreen>>,
}

impl LowcodeStore {
    /// Returns the global in-memory singleton with demo data seeded.
    pub fn global() -> LowcodeStore {
        GLOBAL_STORE
            .get_or_init(|| {
                let s = Self::in_memory();
                s.seed_demo();
                s
            })
            .clone()
    }

    pub fn in_memory() -> Self {
        let mem = Arc::new(MemStore::default());
        Self { db: None, mem }
    }

    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        let database_url = az_aio_platform::core::db::verify_database_url(database_url)?;
        use crate::backend::model::TABLE_NAME_PREFIX;
        let toasty = toasty::Db::builder()
            .models(toasty::models!(
                MetaModel,
                MetaField,
                AppScreen,
                LowcodeRecord
            ))
            .table_name_prefix(TABLE_NAME_PREFIX)
            .connect(database_url)
            .await?;
        toasty.push_schema().await?;
        let db = SharedDb::new(toasty);
        let mem = Arc::new(MemStore::default());
        Ok(Self { db: Some(db), mem })
    }

    pub fn degraded(database_url: Option<String>) -> Self {
        match database_url.filter(|u| !u.trim().is_empty()) {
            Some(url) => match tokio::runtime::Runtime::new() {
                Ok(rt) => match rt.block_on(Self::new(&url)) {
                    Ok(s) => return s,
                    Err(e) => eprintln!("lowcode: DB err ({e}), using memory"),
                },
                Err(e) => eprintln!("lowcode: runtime err ({e}), using memory"),
            },
            None => {}
        }
        Self::in_memory()
    }
}

// Re-export seed so it's accessible
