use toasty::stmt::{List, Query};

use crate::model::{AppScreen, MetaField, MetaFieldView, MetaModel, MetaModelSummary};
use crate::record::RecordStore;

use super::LowcodeStore;

impl LowcodeStore {
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

    pub fn create_model_sync(&self, model: MetaModel) {
        self.mem.models.lock().push(model);
    }

    pub fn create_field_sync(&self, field: &MetaField) {
        self.mem.fields.lock().push(field.clone());
    }

    pub fn delete_model_sync(&self, id: &str) {
        self.mem.models.lock().retain(|m| m.id != id);
        self.mem.fields.lock().retain(|f| f.model_id != id);
        self.mem.screens.lock().retain(|s| s.model_id != id);
        RecordStore::global().delete_model_records(id);
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
            AppScreen::filter(crate::model::AppScreen::fields().model_id().eq(id))
                .delete()
                .exec(&mut *db)
                .await?;
            MetaField::filter(MetaField::fields().model_id().eq(id))
                .delete()
                .exec(&mut *db)
                .await?;
            MetaModel::filter(MetaModel::fields().id().eq(id))
                .delete()
                .exec(&mut *db)
                .await?;
            RecordStore::global().delete_model_records(id);
            return Ok(());
        }
        self.mem.models.lock().retain(|m| m.id != id);
        self.mem.fields.lock().retain(|f| f.model_id != id);
        self.mem.screens.lock().retain(|s| s.model_id != id);
        RecordStore::global().delete_model_records(id);
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
}
