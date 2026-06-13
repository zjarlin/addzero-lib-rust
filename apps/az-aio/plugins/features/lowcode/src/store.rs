#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use toasty::stmt::{List, Query};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    error::{LowcodeError, LowcodeResult},
    model::{LowcodeApp, LowcodeAppSummary, LowcodePage, LowcodePageSummary},
};

const TABLE_NAME_PREFIX: &str = "biz_lowcode_";

#[derive(Clone)]
pub struct LowcodeStore {
    db: Arc<Mutex<toasty::Db>>,
}

impl LowcodeStore {
    pub async fn connect(database_url: &str) -> LowcodeResult<Self> {
        let db = toasty::Db::builder()
            .models(toasty::models!(LowcodeApp, LowcodePage))
            .table_name_prefix(TABLE_NAME_PREFIX)
            .connect(database_url)
            .await?;
        db.push_schema().await?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
        })
    }

    pub async fn list_apps(&self) -> LowcodeResult<Vec<LowcodeAppSummary>> {
        let mut db = self.db.lock().await;
        let apps = Query::<List<LowcodeApp>>::all().exec(&mut *db).await?;
        Ok(apps.into_iter().map(Into::into).collect())
    }

    pub async fn upsert_app(&self, input: LowcodeAppInput) -> LowcodeResult<LowcodeAppSummary> {
        let id = normalized_id(input.id);
        let now = timestamp_string();
        let mut db = self.db.lock().await;
        let existing = Query::<List<LowcodeApp>>::filter(LowcodeApp::fields().id().eq(&id))
            .first()
            .exec(&mut *db)
            .await?;
        let app = match existing {
            Some(_) => {
                LowcodeApp::filter(LowcodeApp::fields().id().eq(&id))
                    .update()
                    .slug(input.slug)
                    .name(input.name)
                    .description(input.description)
                    .enabled(input.enabled)
                    .updated_at(now)
                    .exec(&mut *db)
                    .await?;
                Query::<List<LowcodeApp>>::filter(LowcodeApp::fields().id().eq(&id))
                    .one()
                    .exec(&mut *db)
                    .await?
            }
            None => {
                LowcodeApp::create()
                    .id(id)
                    .slug(input.slug)
                    .name(input.name)
                    .description(input.description)
                    .enabled(input.enabled)
                    .created_at(now.clone())
                    .updated_at(now)
                    .exec(&mut *db)
                    .await?
            }
        };
        Ok(app.into())
    }

    pub async fn list_pages(&self, app_id: &str) -> LowcodeResult<Vec<LowcodePageSummary>> {
        let app_id = normalize_required_id(app_id, LowcodeError::InvalidAppId)?;
        let mut db = self.db.lock().await;
        let pages = Query::<List<LowcodePage>>::filter(LowcodePage::fields().app_id().eq(&app_id))
            .exec(&mut *db)
            .await?;
        Ok(pages.into_iter().map(Into::into).collect())
    }

    pub async fn upsert_page(&self, input: LowcodePageInput) -> LowcodeResult<LowcodePageSummary> {
        let id = normalized_id(input.id);
        let app_id = normalize_required_id(&input.app_id, LowcodeError::InvalidAppId)?;
        let now = timestamp_string();
        let mut db = self.db.lock().await;
        let existing = Query::<List<LowcodePage>>::filter(LowcodePage::fields().id().eq(&id))
            .first()
            .exec(&mut *db)
            .await?;
        let page = match existing {
            Some(_) => {
                LowcodePage::filter(LowcodePage::fields().id().eq(&id))
                    .update()
                    .app_id(app_id)
                    .route(input.route)
                    .title(input.title)
                    .schema_json(input.schema_json)
                    .enabled(input.enabled)
                    .updated_at(now)
                    .exec(&mut *db)
                    .await?;
                Query::<List<LowcodePage>>::filter(LowcodePage::fields().id().eq(&id))
                    .one()
                    .exec(&mut *db)
                    .await?
            }
            None => {
                LowcodePage::create()
                    .id(id)
                    .app_id(app_id)
                    .route(input.route)
                    .title(input.title)
                    .schema_json(input.schema_json)
                    .enabled(input.enabled)
                    .created_at(now.clone())
                    .updated_at(now)
                    .exec(&mut *db)
                    .await?
            }
        };
        Ok(page.into())
    }

    pub async fn delete_page(&self, page_id: &str) -> LowcodeResult<()> {
        let page_id = normalize_required_id(page_id, LowcodeError::InvalidPageId)?;
        let mut db = self.db.lock().await;
        Query::<List<LowcodePage>>::filter(LowcodePage::fields().id().eq(page_id))
            .delete()
            .exec(&mut *db)
            .await?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LowcodeAppInput {
    pub id: Option<String>,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LowcodePageInput {
    pub id: Option<String>,
    pub app_id: String,
    pub route: String,
    pub title: String,
    pub schema_json: String,
    pub enabled: bool,
}

fn normalized_id(value: Option<String>) -> String {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

fn normalize_required_id(value: &str, error: LowcodeError) -> LowcodeResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(error);
    }
    Ok(value.to_string())
}

fn timestamp_string() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    now.as_secs().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_ids_are_non_blank() {
        assert!(!normalized_id(None).is_empty());
    }

    #[test]
    fn provided_ids_are_trimmed() {
        assert_eq!(normalized_id(Some(" page-1 ".to_string())), "page-1");
    }
}
