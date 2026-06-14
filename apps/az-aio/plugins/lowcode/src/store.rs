use std::sync::{Arc, OnceLock};

use az_aio_platform::db::SharedDb;
use parking_lot::Mutex;
use toasty::stmt::{List, Query};

use crate::model::{
    AppScreen, AppScreenSummary, MetaField, MetaFieldView, MetaModel, MetaModelSummary,
};

static GLOBAL_STORE: OnceLock<LowcodeStore> = OnceLock::new();

#[derive(Clone)]
pub struct LowcodeStore {
    db: Option<SharedDb>,
    mem: Arc<MemStore>,
}

#[derive(Default)]
struct MemStore {
    models: Mutex<Vec<MetaModel>>,
    fields: Mutex<Vec<MetaField>>,
    screens: Mutex<Vec<AppScreen>>,
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
        let database_url = az_aio_platform::db::verify_database_url(database_url)?;
        use crate::model::TABLE_NAME_PREFIX;
        let toasty = toasty::Db::builder()
            .models(toasty::models!(MetaModel, MetaField, AppScreen))
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

    // ── Sync accessors (for SSR) ────────────────────────────────

    pub fn list_models_sync(&self) -> Vec<MetaModelSummary> {
        let models = self.mem.models.lock();
        let fields = self.mem.fields.lock();
        models
            .iter()
            .map(|m| {
                let c = fields.iter().filter(|f| f.model_id == m.id).count() as i64;
                MetaModelSummary {
                    id: m.id.clone(),
                    name: m.name.clone(),
                    label: m.label.clone(),
                    description: m.description.clone(),
                    field_count: c,
                }
            })
            .collect()
    }

    pub fn list_fields_sync(&self, model_id: &str) -> Vec<MetaFieldView> {
        let fields = self.mem.fields.lock();
        let models = self.mem.models.lock();
        let mut views: Vec<MetaFieldView> = fields
            .iter()
            .filter(|f| f.model_id == model_id)
            .map(|f| {
                let mut v = MetaFieldView::from(f.clone());
                if let Some(ref rel_id) = f.relation_model_id {
                    v.relation_model_name = models
                        .iter()
                        .find(|m| m.id == *rel_id)
                        .map(|m| m.name.clone());
                }
                v
            })
            .collect();
        views.sort_by_key(|v| v.order);
        views
    }

    pub fn list_screens_sync(&self) -> Vec<AppScreenSummary> {
        let screens = self.mem.screens.lock();
        let models = self.mem.models.lock();
        screens
            .iter()
            .map(|s| {
                let mn = models
                    .iter()
                    .find(|m| m.id == s.model_id)
                    .map(|m| m.name.clone())
                    .unwrap_or_default();
                AppScreenSummary {
                    id: s.id.clone(),
                    name: s.name.clone(),
                    label: s.label.clone(),
                    layout: s.layout.clone(),
                    model_id: s.model_id.clone(),
                    model_name: mn,
                    created_at: s.created_at.clone(),
                }
            })
            .collect()
    }


    pub fn create_model_sync(&self, model: MetaModel) {
        self.mem.models.lock().push(model);
    }

    pub fn create_field_sync(&self, field: &MetaField) {
        self.mem.fields.lock().push(field.clone());
    }


    pub fn delete_model_sync(&self, id: &str) {
        self.mem.models.lock().retain(|m| m.id != id);
        self.mem.fields.lock().retain(|f| f.model_id != id);
        // Cascade: remove all runtime records for this model
        crate::record::RecordStore::global().delete_model_records(id);
    }

    pub fn mem_fields_sync(&self) -> Vec<MetaField> {
        self.mem.fields.lock().clone()
    }

    pub fn update_field_sync_v(&self, field: &MetaField) {
        let mut fields = self.mem.fields.lock();
        if let Some(f) = fields.iter_mut().find(|f| f.id == field.id) {
            *f = field.clone();
        }
    }

    pub fn delete_field_sync(&self, id: &str) {
        self.mem.fields.lock().retain(|f| f.id != id);
    }

    pub fn get_screen_sync(&self, id: &str) -> Option<AppScreen> {
        self.mem.screens.lock().iter().find(|s| s.id == id).cloned()
    }

    // ── Async MetaModel CRUD ────────────────────────────────────

    pub async fn list_models(&self) -> anyhow::Result<Vec<MetaModelSummary>> {
        if let Some(ref db) = self.db {
            let mut db = db.lock().await;
            let models = Query::<List<MetaModel>>::all().exec(&mut *db).await?;
            let mut out = Vec::with_capacity(models.len());
            for m in models {
                let c = Query::<List<MetaField>>::filter(
                    MetaField::fields().model_id().eq(&m.id),
                )
                .exec(&mut *db)
                .await?
                .len() as i64;
                out.push(MetaModelSummary {
                    id: m.id,
                    name: m.name,
                    label: m.label,
                    description: m.description,
                    field_count: c,
                });
            }
            return Ok(out);
        }
        Ok(self.list_models_sync())
    }

    pub async fn get_model(&self, id: &str) -> anyhow::Result<Option<MetaModel>> {
        if let Some(ref db) = self.db {
            let mut db = db.lock().await;
            return Query::<List<MetaModel>>::filter(MetaModel::fields().id().eq(id))
                .first()
                .exec(&mut *db)
                .await
                .map_err(Into::into);
        }
        Ok(self.mem.models.lock().iter().find(|m| m.id == id).cloned())
    }

    pub async fn create_model(&self, model: MetaModel) -> anyhow::Result<MetaModel> {
        if let Some(ref db) = self.db {
            let mut db = db.lock().await;
            return MetaModel::create()
                .id(&model.id)
                .name(&model.name)
                .label(&model.label)
                .description(&model.description)
                .created_at(&model.created_at)
                .updated_at(&model.updated_at)
                .exec(&mut *db)
                .await
                .map_err(Into::into);
        }
        self.mem.models.lock().push(model.clone());
        Ok(model)
    }

    pub async fn update_model(&self, model: &MetaModel) -> anyhow::Result<()> {
        if let Some(ref db) = self.db {
            let mut db = db.lock().await;
            MetaModel::filter(MetaModel::fields().id().eq(&model.id))
                .update()
                .name(&model.name)
                .label(&model.label)
                .description(&model.description)
                .updated_at(&model.updated_at)
                .exec(&mut *db)
                .await?;
            return Ok(());
        }
        let mut models = self.mem.models.lock();
        if let Some(m) = models.iter_mut().find(|m| m.id == model.id) {
            *m = model.clone();
        }
        Ok(())
    }

    pub async fn delete_model(&self, id: &str) -> anyhow::Result<()> {
        if let Some(ref db) = self.db {
            let mut db = db.lock().await;
            MetaField::filter(MetaField::fields().model_id().eq(id))
                .delete()
                .exec(&mut *db)
                .await?;
            MetaModel::filter(MetaModel::fields().id().eq(id))
                .delete()
                .exec(&mut *db)
                .await?;
            crate::record::RecordStore::global().delete_model_records(id);
            return Ok(());
        }
        self.mem.models.lock().retain(|m| m.id != id);
        self.mem.fields.lock().retain(|f| f.model_id != id);
        crate::record::RecordStore::global().delete_model_records(id);
        Ok(())
    }

    // ── Async MetaField CRUD ────────────────────────────────────

    pub async fn list_fields(&self, model_id: &str) -> anyhow::Result<Vec<MetaFieldView>> {
        if let Some(ref db) = self.db {
            let mut db = db.lock().await;
            let fields = Query::<List<MetaField>>::filter(
                MetaField::fields().model_id().eq(model_id),
            )
            .exec(&mut *db)
            .await?;
            let mut views: Vec<MetaFieldView> =
                fields.into_iter().map(MetaFieldView::from).collect();
            for v in &mut views {
                if let Some(ref rel_id) = v.relation_model_id {
                    if let Ok(Some(m)) =
                        Query::<List<MetaModel>>::filter(MetaModel::fields().id().eq(rel_id.as_str()))
                            .first()
                            .exec(&mut *db)
                            .await
                    {
                        v.relation_model_name = Some(m.name);
                    }
                }
            }
            return Ok(views);
        }
        Ok(self.list_fields_sync(model_id))
    }

    pub async fn get_field(&self, id: &str) -> anyhow::Result<Option<MetaField>> {
        if let Some(ref db) = self.db {
            let mut db = db.lock().await;
            return Query::<List<MetaField>>::filter(MetaField::fields().id().eq(id))
                .first()
                .exec(&mut *db)
                .await
                .map_err(Into::into);
        }
        Ok(self.mem.fields.lock().iter().find(|f| f.id == id).cloned())
    }

    pub async fn create_field(&self, field: &MetaField) -> anyhow::Result<MetaField> {
        if let Some(ref db) = self.db {
            let mut db = db.lock().await;
            return MetaField::create()
                .id(&field.id)
                .model_id(&field.model_id)
                .name(&field.name)
                .label(&field.label)
                .field_type(&field.field_type)
                .relation_type(field.relation_type.clone())
                .relation_model_id(field.relation_model_id.clone())
                .is_required(field.is_required)
                .is_unique(field.is_unique)
                .order(field.order)
                .default_value(field.default_value.clone())
                .created_at(&field.created_at)
                .updated_at(&field.updated_at)
                .exec(&mut *db)
                .await
                .map_err(Into::into);
        }
        self.mem.fields.lock().push(field.clone());
        Ok(field.clone())
    }

    pub async fn update_field(&self, field: &MetaField) -> anyhow::Result<()> {
        if let Some(ref db) = self.db {
            let mut db = db.lock().await;
            MetaField::filter(MetaField::fields().id().eq(&field.id))
                .update()
                .name(&field.name)
                .label(&field.label)
                .field_type(&field.field_type)
                .relation_type(field.relation_type.clone())
                .relation_model_id(field.relation_model_id.clone())
                .is_required(field.is_required)
                .is_unique(field.is_unique)
                .order(field.order)
                .default_value(field.default_value.clone())
                .updated_at(&field.updated_at)
                .exec(&mut *db)
                .await?;
            return Ok(());
        }
        let mut fields = self.mem.fields.lock();
        if let Some(f) = fields.iter_mut().find(|f| f.id == field.id) {
            *f = field.clone();
        }
        Ok(())
    }

    pub async fn delete_field(&self, id: &str) -> anyhow::Result<()> {
        if let Some(ref db) = self.db {
            let mut db = db.lock().await;
            MetaField::filter(MetaField::fields().id().eq(id))
                .delete()
                .exec(&mut *db)
                .await?;
            return Ok(());
        }
        self.mem.fields.lock().retain(|f| f.id != id);
        Ok(())
    }

    // ── Async AppScreen CRUD ────────────────────────────────────

    pub async fn list_screens(&self) -> anyhow::Result<Vec<AppScreenSummary>> {
        if let Some(ref db) = self.db {
            let mut db = db.lock().await;
            let screens = Query::<List<AppScreen>>::all().exec(&mut *db).await?;
            let mut out = Vec::with_capacity(screens.len());
            for s in screens {
                let mn = Query::<List<MetaModel>>::filter(
                    MetaModel::fields().id().eq(&s.model_id),
                )
                .first()
                .exec(&mut *db)
                .await?
                .map(|m| m.name)
                .unwrap_or_default();
                out.push(AppScreenSummary {
                    id: s.id,
                    name: s.name,
                    label: s.label,
                    layout: s.layout,
                    model_id: s.model_id,
                    model_name: mn,
                    created_at: s.created_at,
                });
            }
            return Ok(out);
        }
        Ok(self.list_screens_sync())
    }

    pub async fn get_screen(&self, id: &str) -> anyhow::Result<Option<AppScreen>> {
        if let Some(ref db) = self.db {
            let mut db = db.lock().await;
            return Query::<List<AppScreen>>::filter(AppScreen::fields().id().eq(id))
                .first()
                .exec(&mut *db)
                .await
                .map_err(Into::into);
        }
        Ok(self.get_screen_sync(id))
    }

    pub async fn create_screen(&self, screen: &AppScreen) -> anyhow::Result<AppScreen> {
        if let Some(ref db) = self.db {
            let mut db = db.lock().await;
            return AppScreen::create()
                .id(&screen.id)
                .name(&screen.name)
                .label(&screen.label)
                .layout(&screen.layout)
                .model_id(&screen.model_id)
                .config_json(&screen.config_json)
                .created_at(&screen.created_at)
                .updated_at(&screen.updated_at)
                .exec(&mut *db)
                .await
                .map_err(Into::into);
        }
        self.mem.screens.lock().push(screen.clone());
        Ok(screen.clone())
    }

    pub async fn delete_screen(&self, id: &str) -> anyhow::Result<()> {
        if let Some(ref db) = self.db {
            let mut db = db.lock().await;
            AppScreen::filter(AppScreen::fields().id().eq(id))
                .delete()
                .exec(&mut *db)
                .await?;
            return Ok(());
        }
        self.mem.screens.lock().retain(|s| s.id != id);
        Ok(())
    }

    // ── Seed demo data ──────────────────────────────────────────


    pub fn create_screen_sync(&self, screen: AppScreen) {
        self.mem.screens.lock().push(screen);
    }


    pub fn update_screen_label_sync(&self, id: &str, label: &str) {
        let mut screens = self.mem.screens.lock();
        if let Some(s) = screens.iter_mut().find(|s| s.id == id) {
            s.label = label.to_string();
        }
    }

    pub fn delete_screen_sync(&self, id: &str) {
        self.mem.screens.lock().retain(|s| s.id != id);
    }

    pub fn seed_demo(&self) {
        let now = chrono::Utc::now().to_rfc3339();
        let mut models = self.mem.models.lock();
        if !models.is_empty() {
            return;
        }
        let mut fields = self.mem.fields.lock();

        let proj_id = "demo-proj-001".to_string();
        models.push(MetaModel {
            id: proj_id.clone(),
            name: "Project".into(),
            label: "项目".into(),
            description: "项目管理模型".into(),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-001".into(),
            model_id: proj_id.clone(),
            name: "name".into(),
            label: "名称".into(),
            field_type: "String".into(),
            relation_type: None,
            relation_model_id: None,
            is_required: true,
            is_unique: false,
            order: 1,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-002".into(),
            model_id: proj_id.clone(),
            name: "status".into(),
            label: "状态".into(),
            field_type: "String".into(),
            relation_type: None,
            relation_model_id: None,
            is_required: false,
            is_unique: false,
            order: 2,
            default_value: Some("draft".into()),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-003".into(),
            model_id: proj_id.clone(),
            name: "start_date".into(),
            label: "开始日期".into(),
            field_type: "DateTime".into(),
            relation_type: None,
            relation_model_id: None,
            is_required: false,
            is_unique: false,
            order: 3,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-004".into(),
            model_id: proj_id.clone(),
            name: "budget".into(),
            label: "预算".into(),
            field_type: "Float".into(),
            relation_type: None,
            relation_model_id: None,
            is_required: false,
            is_unique: false,
            order: 4,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });

        let emp_id = "demo-emp-001".to_string();
        models.push(MetaModel {
            id: emp_id.clone(),
            name: "Employee".into(),
            label: "员工".into(),
            description: "员工信息模型".into(),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-010".into(),
            model_id: emp_id.clone(),
            name: "name".into(),
            label: "姓名".into(),
            field_type: "String".into(),
            relation_type: None,
            relation_model_id: None,
            is_required: true,
            is_unique: false,
            order: 1,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-011".into(),
            model_id: emp_id.clone(),
            name: "email".into(),
            label: "邮箱".into(),
            field_type: "String".into(),
            relation_type: None,
            relation_model_id: None,
            is_required: false,
            is_unique: true,
            order: 2,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-012".into(),
            model_id: emp_id.clone(),
            name: "project_id".into(),
            label: "所属项目".into(),
            field_type: "Relation".into(),
            relation_type: Some("OneToMany".into()),
            relation_model_id: Some(proj_id.clone()),
            is_required: false,
            is_unique: false,
            order: 3,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-013".into(),
            model_id: emp_id.clone(),
            name: "manager_id".into(),
            label: "上级主管".into(),
            field_type: "Relation".into(),
            relation_type: Some("SelfRecursive".into()),
            relation_model_id: Some(emp_id.clone()),
            is_required: false,
            is_unique: false,
            order: 4,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });

        let org_id = "demo-org-001".to_string();
        models.push(MetaModel {
            id: org_id.clone(),
            name: "Organization".into(),
            label: "组织架构".into(),
            description: "树形组织架构".into(),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-020".into(),
            model_id: org_id.clone(),
            name: "name".into(),
            label: "名称".into(),
            field_type: "String".into(),
            relation_type: None,
            relation_model_id: None,
            is_required: true,
            is_unique: false,
            order: 1,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-021".into(),
            model_id: org_id.clone(),
            name: "parent_id".into(),
            label: "上级部门".into(),
            field_type: "Relation".into(),
            relation_type: Some("SelfRecursive".into()),
            relation_model_id: Some(org_id.clone()),
            is_required: false,
            is_unique: false,
            order: 2,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        fields.push(MetaField {
            id: "demo-f-022".into(),
            model_id: org_id.clone(),
            name: "level".into(),
            label: "层级".into(),
            field_type: "Integer".into(),
            relation_type: None,
            relation_model_id: None,
            is_required: false,
            is_unique: false,
            order: 3,
            default_value: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        });

        let mut screens = self.mem.screens.lock();
        screens.push(AppScreen {
            id: "demo-screen-table".into(),
            name: "project-table".into(),
            label: "项目列表".into(),
            layout: "Table".into(),
            model_id: proj_id.clone(),
            config_json: r#"{"columns":[{"field_name":"name","label":"名称","sortable":true},{"field_name":"status","label":"状态"},{"field_name":"start_date","label":"开始日期","sortable":true},{"field_name":"budget","label":"预算"}],"searchable_fields":["name"],"page_size":20}"#.into(),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        screens.push(AppScreen {
            id: "demo-screen-md".into(),
            name: "project-org".into(),
            label: "项目-组织左树右表".into(),
            layout: "MasterDetail".into(),
            model_id: org_id.clone(),
            config_json: r#"{"tree_field_id":"name","detail_columns":[{"field_name":"name","label":"名称","sortable":true},{"field_name":"level","label":"层级"},{"field_name":"parent_id","label":"上级部门"}],"detail_searchable":["name"]}"#.into(),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        screens.push(AppScreen {
            id: "demo-screen-acc".into(),
            name: "employee-detail".into(),
            label: "员工详情手风琴".into(),
            layout: "Accordion".into(),
            model_id: emp_id.clone(),
            config_json: r#"{"groups":[{"label":"基本信息","fields":["name","email"]},{"label":"组织关系","fields":["project_id","manager_id"]}]}"#.into(),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        screens.push(AppScreen {
            id: "demo-screen-form".into(),
            name: "employee-form".into(),
            label: "员工录入表单".into(),
            layout: "Form".into(),
            model_id: emp_id.clone(),
            config_json: r#"{"fields":[{"field_name":"name","label":"姓名","field_type":"string","required":true,"placeholder":"输入员工姓名"},{"field_name":"email","label":"邮箱","field_type":"string","required":false,"placeholder":"email@example.com"},{"field_name":"project_id","label":"所属项目","field_type":"string","required":false,"placeholder":"选择项目"}],"submit_label":"保存"}"#.into(),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
        screens.push(AppScreen {
            id: "demo-screen-tree".into(),
            name: "org-tree".into(),
            label: "组织架构树".into(),
            layout: "TreeTable".into(),
            model_id: org_id.clone(),
            config_json: r#"{"tree_field":"parent_id","label_field":"name","columns":[{"field_name":"name","label":"名称"},{"field_name":"level","label":"层级"}]}"#.into(),
            created_at: now.clone(),
            updated_at: now.clone(),
        });
    }
}
