use toasty::stmt::{List, Query};

use crate::model::{AppScreen, AppScreenSummary, MetaModel};

use super::LowcodeStore;

impl LowcodeStore {
    // ── Sync accessors ──────────────────────────────────────────

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

    pub fn get_screen_sync(&self, id: &str) -> Option<AppScreen> {
        self.mem
            .screens
            .lock()
            .iter()
            .find(|s| s.id == id)
            .cloned()
    }

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
}
