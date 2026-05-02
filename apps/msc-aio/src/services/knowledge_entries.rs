use std::{future::Future, pin::Pin, rc::Rc};

use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use addzero_knowledge::{KnowledgeDocument, KnowledgeService, ManualKnowledgeDocumentInput};

pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeEntryUpsertDto {
    pub source_slug: String,
    pub source_name: String,
    pub source_root: String,
    pub source_path: String,
    pub relative_path: String,
    pub title: String,
    pub source_label: String,
    pub body: String,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeEntrySavedDto {
    pub slug: String,
    pub title: String,
    pub source_path: String,
    pub relative_path: String,
    pub section_count: usize,
}

pub trait KnowledgeEntriesApi: 'static {
    fn save_entry(
        &self,
        input: KnowledgeEntryUpsertDto,
    ) -> LocalBoxFuture<'_, Result<KnowledgeEntrySavedDto, String>>;
}

pub type SharedKnowledgeEntriesApi = Rc<dyn KnowledgeEntriesApi>;

pub fn default_knowledge_entries_api() -> SharedKnowledgeEntriesApi {
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(BrowserKnowledgeEntriesApi)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(EmbeddedKnowledgeEntriesApi)
    }
}

#[cfg(target_arch = "wasm32")]
struct BrowserKnowledgeEntriesApi;

#[cfg(target_arch = "wasm32")]
impl KnowledgeEntriesApi for BrowserKnowledgeEntriesApi {
    fn save_entry(
        &self,
        input: KnowledgeEntryUpsertDto,
    ) -> LocalBoxFuture<'_, Result<KnowledgeEntrySavedDto, String>> {
        Box::pin(
            async move { super::browser_http::post_json("/api/knowledge/entries", &input).await },
        )
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct EmbeddedKnowledgeEntriesApi;

#[cfg(not(target_arch = "wasm32"))]
impl KnowledgeEntriesApi for EmbeddedKnowledgeEntriesApi {
    fn save_entry(
        &self,
        input: KnowledgeEntryUpsertDto,
    ) -> LocalBoxFuture<'_, Result<KnowledgeEntrySavedDto, String>> {
        Box::pin(async move { save_knowledge_entry_on_server(input).await })
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn save_knowledge_entry_on_server(
    input: KnowledgeEntryUpsertDto,
) -> Result<KnowledgeEntrySavedDto, String> {
    let database_url = addzero_knowledge::database_url().ok_or_else(|| {
        "缺少 PostgreSQL 连接：请设置 MSC_AIO_DATABASE_URL 或 DATABASE_URL".to_string()
    })?;
    let service = KnowledgeService::connect(&database_url)
        .await
        .map_err(|err| format!("连接知识库失败：{err}"))?;
    let saved = service
        .upsert_manual_document(ManualKnowledgeDocumentInput {
            source_slug: input.source_slug,
            source_name: input.source_name,
            source_root: input.source_root,
            source_path: input.source_path,
            relative_path: input.relative_path,
            title: input.title,
            source_label: input.source_label,
            body: input.body,
            tags: input.tags,
        })
        .await
        .map_err(|err| format!("写入知识库失败：{err}"))?;
    Ok(saved_to_dto(saved))
}

#[cfg(not(target_arch = "wasm32"))]
fn saved_to_dto(saved: KnowledgeDocument) -> KnowledgeEntrySavedDto {
    KnowledgeEntrySavedDto {
        slug: saved.slug,
        title: saved.title,
        source_path: saved.source_path,
        relative_path: saved.relative_path,
        section_count: saved.section_count,
    }
}
