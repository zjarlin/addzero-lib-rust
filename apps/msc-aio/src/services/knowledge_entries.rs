use std::{future::Future, pin::Pin, rc::Rc};

use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use addzero_knowledge::{KnowledgeDocument, KnowledgeService, ManualKnowledgeDocumentInput};
#[cfg(not(target_arch = "wasm32"))]
use chrono::Utc;

pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub const WORKSPACE_SOURCE_SLUG: &str = "workspace-notes";
pub const WORKSPACE_SOURCE_NAME: &str = "笔记工作台";
pub const WORKSPACE_SOURCE_ROOT: &str = "msc-aio://notes";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeNoteDto {
    pub slug: String,
    pub title: String,
    pub filename: String,
    pub source_path: String,
    pub relative_path: String,
    pub preview: String,
    pub excerpt: String,
    pub headings: Vec<String>,
    pub body: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeEntryUpsertDto {
    pub source_path: String,
    pub relative_path: String,
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeEntryDeleteDto {
    pub source_path: String,
}

pub trait KnowledgeEntriesApi: 'static {
    fn list_entries(&self) -> LocalBoxFuture<'_, Result<Vec<KnowledgeNoteDto>, String>>;

    fn save_entry(
        &self,
        input: KnowledgeEntryUpsertDto,
    ) -> LocalBoxFuture<'_, Result<KnowledgeNoteDto, String>>;

    fn delete_entry(
        &self,
        input: KnowledgeEntryDeleteDto,
    ) -> LocalBoxFuture<'_, Result<(), String>>;
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
    fn list_entries(&self) -> LocalBoxFuture<'_, Result<Vec<KnowledgeNoteDto>, String>> {
        Box::pin(async move { super::browser_http::get_json("/api/knowledge/entries").await })
    }

    fn save_entry(
        &self,
        input: KnowledgeEntryUpsertDto,
    ) -> LocalBoxFuture<'_, Result<KnowledgeNoteDto, String>> {
        Box::pin(
            async move { super::browser_http::post_json("/api/knowledge/entries", &input).await },
        )
    }

    fn delete_entry(
        &self,
        input: KnowledgeEntryDeleteDto,
    ) -> LocalBoxFuture<'_, Result<(), String>> {
        Box::pin(async move {
            let _: serde_json::Value =
                super::browser_http::post_json("/api/knowledge/entries/delete", &input).await?;
            Ok(())
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct EmbeddedKnowledgeEntriesApi;

#[cfg(not(target_arch = "wasm32"))]
impl KnowledgeEntriesApi for EmbeddedKnowledgeEntriesApi {
    fn list_entries(&self) -> LocalBoxFuture<'_, Result<Vec<KnowledgeNoteDto>, String>> {
        Box::pin(async move { list_knowledge_entries_on_server().await })
    }

    fn save_entry(
        &self,
        input: KnowledgeEntryUpsertDto,
    ) -> LocalBoxFuture<'_, Result<KnowledgeNoteDto, String>> {
        Box::pin(async move { save_knowledge_entry_on_server(input).await })
    }

    fn delete_entry(
        &self,
        input: KnowledgeEntryDeleteDto,
    ) -> LocalBoxFuture<'_, Result<(), String>> {
        Box::pin(async move { delete_knowledge_entry_on_server(input).await })
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn list_knowledge_entries_on_server() -> Result<Vec<KnowledgeNoteDto>, String> {
    let service = connect_knowledge_service().await?;
    let mut notes = service
        .list_documents()
        .await
        .map_err(|err| format!("读取笔记失败：{err}"))?
        .into_iter()
        .filter(|doc| doc.source_slug == WORKSPACE_SOURCE_SLUG)
        .map(document_to_dto)
        .collect::<Vec<_>>();
    notes.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(notes)
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn save_knowledge_entry_on_server(
    input: KnowledgeEntryUpsertDto,
) -> Result<KnowledgeNoteDto, String> {
    let service = connect_knowledge_service().await?;
    let title = derive_title(input.title.as_str(), input.body.as_str());
    let (relative_path, source_path) = normalize_paths(&input, &title);
    let saved = service
        .upsert_manual_document(ManualKnowledgeDocumentInput {
            source_slug: WORKSPACE_SOURCE_SLUG.to_string(),
            source_name: WORKSPACE_SOURCE_NAME.to_string(),
            source_root: WORKSPACE_SOURCE_ROOT.to_string(),
            source_path,
            relative_path,
            title,
            source_label: WORKSPACE_SOURCE_NAME.to_string(),
            body: input.body,
            tags: input.tags,
        })
        .await
        .map_err(|err| format!("写入笔记失败：{err}"))?;
    Ok(document_to_dto(saved))
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn delete_knowledge_entry_on_server(
    input: KnowledgeEntryDeleteDto,
) -> Result<(), String> {
    let source_path = input.source_path.trim();
    if source_path.is_empty() {
        return Ok(());
    }

    let service = connect_knowledge_service().await?;
    service
        .delete_document_by_source_path(source_path)
        .await
        .map_err(|err| format!("删除笔记失败：{err}"))?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
async fn connect_knowledge_service() -> Result<KnowledgeService, String> {
    let database_url = addzero_knowledge::database_url().ok_or_else(|| {
        "缺少 PostgreSQL 连接：请设置 MSC_AIO_DATABASE_URL 或 DATABASE_URL".to_string()
    })?;
    KnowledgeService::connect(&database_url)
        .await
        .map_err(|err| format!("连接知识库失败：{err}"))
}

#[cfg(not(target_arch = "wasm32"))]
fn document_to_dto(saved: KnowledgeDocument) -> KnowledgeNoteDto {
    KnowledgeNoteDto {
        slug: saved.slug,
        title: saved.title,
        filename: saved.filename,
        source_path: saved.source_path,
        relative_path: saved.relative_path,
        preview: saved.preview,
        excerpt: saved.excerpt,
        headings: saved.headings,
        body: saved.body,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn normalize_paths(input: &KnowledgeEntryUpsertDto, title: &str) -> (String, String) {
    let relative_path = if !input.relative_path.trim().is_empty() {
        input
            .relative_path
            .trim()
            .trim_start_matches('/')
            .to_string()
    } else if !input.source_path.trim().is_empty() {
        input
            .source_path
            .trim()
            .trim_start_matches(WORKSPACE_SOURCE_ROOT)
            .trim_start_matches('/')
            .to_string()
    } else {
        format!("{}.md", generated_note_stem(title))
    };

    let source_path = if !input.source_path.trim().is_empty() {
        input.source_path.trim().to_string()
    } else {
        format!("{WORKSPACE_SOURCE_ROOT}/{relative_path}")
    };

    (relative_path, source_path)
}

#[cfg(not(target_arch = "wasm32"))]
fn generated_note_stem(title: &str) -> String {
    let slug = slugify(title);
    let prefix = if slug.is_empty() {
        "note"
    } else {
        slug.as_str()
    };
    format!("{prefix}-{}", Utc::now().format("%Y%m%d%H%M%S"))
}

#[cfg(not(target_arch = "wasm32"))]
fn derive_title(title: &str, body: &str) -> String {
    let trimmed = title.trim();
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }

    if let Some(heading) = body
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with('#') && line.trim_start_matches('#').trim().len() > 0)
    {
        return heading.trim_start_matches('#').trim().to_string();
    }

    if let Some(line) = body.lines().map(str::trim).find(|line| !line.is_empty()) {
        return line
            .trim_start_matches(['#', '-', '*', ' '])
            .chars()
            .take(48)
            .collect::<String>();
    }

    "新笔记".to_string()
}

#[cfg(not(target_arch = "wasm32"))]
fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;

    for ch in value.chars() {
        let lowered = ch.to_ascii_lowercase();
        if lowered.is_ascii_alphanumeric() {
            slug.push(lowered);
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }

    slug.trim_matches('-').to_string()
}
