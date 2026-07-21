use std::collections::{BTreeMap, HashSet};

use anyhow::{Context, anyhow};
use az_persistence::context::PersistenceDb;
use chrono::{DateTime, Utc};
use jiff::Timestamp;
use toasty::stmt::{List, Query};
use uuid::Uuid;

use crate::{
    models::{
        knowledge_document::KnowledgeDocumentRecord, knowledge_source::KnowledgeSourceRecord,
    },
    types::{KnowledgeDocument, KnowledgeSourceSpec},
};

#[derive(Clone)]
pub(crate) struct KnowledgeRepository {
    db: PersistenceDb,
}

impl KnowledgeRepository {
    pub(crate) fn new(db: PersistenceDb) -> Self {
        Self { db }
    }

    pub(crate) async fn list_documents(&self) -> anyhow::Result<Vec<KnowledgeDocument>> {
        let mut db = self.db.lock().await;
        let sources = Query::<List<KnowledgeSourceRecord>>::all()
            .exec(&mut *db)
            .await
            .context("查询知识源失败")?;
        let documents = Query::<List<KnowledgeDocumentRecord>>::filter(
            KnowledgeDocumentRecord::fields().is_active().eq(true),
        )
        .exec(&mut *db)
        .await
        .context("查询知识文档失败")?;
        let source_map = sources
            .into_iter()
            .map(|source| (source.id, source))
            .collect::<BTreeMap<_, _>>();
        let mut result = documents
            .into_iter()
            .map(|document| document_from_record(document, &source_map))
            .collect::<anyhow::Result<Vec<_>>>()?;
        result.sort_by(|left, right| {
            left.source_name
                .cmp(&right.source_name)
                .then(left.title.cmp(&right.title))
                .then(left.source_path.cmp(&right.source_path))
        });
        Ok(result)
    }

    pub(crate) async fn upsert_source(
        &self,
        source: &KnowledgeSourceSpec,
    ) -> anyhow::Result<Uuid> {
        let now = Timestamp::now();
        let mut db = self.db.lock().await;
        let existing = Query::<List<KnowledgeSourceRecord>>::filter(
            KnowledgeSourceRecord::fields().slug().eq(&source.slug),
        )
        .first()
        .exec(&mut *db)
        .await
        .with_context(|| format!("查询知识源失败: {}", source.slug))?;
        match existing {
            Some(existing) => {
                KnowledgeSourceRecord::filter(
                    KnowledgeSourceRecord::fields().id().eq(existing.id),
                )
                .update()
                .name(&source.name)
                .root_path(source.root_path.display().to_string())
                .last_synced_at(Some(now))
                .updated_at(now)
                .exec(&mut *db)
                .await
                .with_context(|| format!("更新知识源失败: {}", source.slug))?;
                Ok(existing.id)
            }
            None => {
                let id = Uuid::new_v4();
                KnowledgeSourceRecord::create()
                    .id(id)
                    .slug(&source.slug)
                    .name(&source.name)
                    .root_path(source.root_path.display().to_string())
                    .last_synced_at(Some(now))
                    .created_at(now)
                    .updated_at(now)
                    .exec(&mut *db)
                    .await
                    .with_context(|| format!("创建知识源失败: {}", source.slug))?;
                Ok(id)
            }
        }
    }

    pub(crate) async fn upsert_document(
        &self,
        source_id: Uuid,
        document: &KnowledgeDocument,
    ) -> anyhow::Result<()> {
        let now = Timestamp::now();
        let updated_at = jiff_timestamp(document.updated_at)?;
        let mut db = self.db.lock().await;
        let existing = Query::<List<KnowledgeDocumentRecord>>::filter(
            KnowledgeDocumentRecord::fields()
                .source_path()
                .eq(&document.source_path),
        )
        .first()
        .exec(&mut *db)
        .await
        .with_context(|| format!("查询知识文档失败: {}", document.source_path))?;
        match existing {
            Some(existing) => {
                KnowledgeDocumentRecord::filter(
                    KnowledgeDocumentRecord::fields().id().eq(existing.id),
                )
                .update()
                .source_id(source_id)
                .slug(&document.slug)
                .title(&document.title)
                .filename(&document.filename)
                .relative_path(&document.relative_path)
                .bytes(i64::try_from(document.bytes).unwrap_or(i64::MAX))
                .section_count(i32::try_from(document.section_count).unwrap_or(i32::MAX))
                .preview(&document.preview)
                .excerpt(&document.excerpt)
                .headings(document.headings.clone())
                .tags(document.tags.clone())
                .body(&document.body)
                .content_hash(&document.content_hash)
                .is_active(true)
                .updated_at(updated_at)
                .exec(&mut *db)
                .await
                .with_context(|| format!("更新知识文档失败: {}", document.source_path))?;
            }
            None => {
                KnowledgeDocumentRecord::create()
                    .id(Uuid::new_v4())
                    .source_id(source_id)
                    .slug(&document.slug)
                    .title(&document.title)
                    .filename(&document.filename)
                    .source_path(&document.source_path)
                    .relative_path(&document.relative_path)
                    .bytes(i64::try_from(document.bytes).unwrap_or(i64::MAX))
                    .section_count(i32::try_from(document.section_count).unwrap_or(i32::MAX))
                    .preview(&document.preview)
                    .excerpt(&document.excerpt)
                    .headings(document.headings.clone())
                    .tags(document.tags.clone())
                    .body(&document.body)
                    .content_hash(&document.content_hash)
                    .is_active(true)
                    .created_at(now)
                    .updated_at(updated_at)
                    .exec(&mut *db)
                    .await
                    .with_context(|| format!("创建知识文档失败: {}", document.source_path))?;
            }
        }
        Ok(())
    }

    pub(crate) async fn source_by_slug(
        &self,
        slug: &str,
    ) -> anyhow::Result<Option<KnowledgeSourceRecord>> {
        let mut db = self.db.lock().await;
        Query::<List<KnowledgeSourceRecord>>::filter(
            KnowledgeSourceRecord::fields().slug().eq(slug),
        )
        .first()
        .exec(&mut *db)
        .await
        .with_context(|| format!("查询知识源失败: {slug}"))
    }

    pub(crate) async fn deactivate_missing_documents(
        &self,
        source_id: Uuid,
        active_paths: &[String],
    ) -> anyhow::Result<()> {
        let active_paths = active_paths.iter().collect::<HashSet<_>>();
        let mut db = self.db.lock().await;
        let records = Query::<List<KnowledgeDocumentRecord>>::filter(
            KnowledgeDocumentRecord::fields()
                .source_id()
                .eq(source_id)
                .and(KnowledgeDocumentRecord::fields().is_active().eq(true)),
        )
        .exec(&mut *db)
        .await
        .context("查询待停用知识文档失败")?;
        let now = Timestamp::now();
        for record in records {
            if active_paths.contains(&record.source_path) {
                continue;
            }
            KnowledgeDocumentRecord::filter(
                KnowledgeDocumentRecord::fields().id().eq(record.id),
            )
            .update()
            .is_active(false)
            .updated_at(now)
            .exec(&mut *db)
            .await
            .with_context(|| format!("停用知识文档失败: {}", record.source_path))?;
        }
        Ok(())
    }

    pub(crate) async fn deactivate_document_by_source_path(
        &self,
        source_path: &str,
    ) -> anyhow::Result<()> {
        let mut db = self.db.lock().await;
        KnowledgeDocumentRecord::filter(
            KnowledgeDocumentRecord::fields()
                .source_path()
                .eq(source_path),
        )
        .update()
        .is_active(false)
        .updated_at(Timestamp::now())
        .exec(&mut *db)
        .await
        .with_context(|| format!("停用知识文档失败: {source_path}"))?;
        Ok(())
    }
}

fn document_from_record(
    document: KnowledgeDocumentRecord,
    sources: &BTreeMap<Uuid, KnowledgeSourceRecord>,
) -> anyhow::Result<KnowledgeDocument> {
    let source = sources.get(&document.source_id).ok_or_else(|| {
        anyhow!("知识文档缺少对应知识源: {}", document.source_path)
    })?;
    Ok(KnowledgeDocument {
        source_slug: source.slug.clone(),
        source_name: source.name.clone(),
        source_root: source.root_path.clone(),
        slug: document.slug,
        title: document.title,
        filename: document.filename,
        source_path: document.source_path,
        relative_path: document.relative_path,
        bytes: usize::try_from(document.bytes).unwrap_or_default(),
        section_count: usize::try_from(document.section_count).unwrap_or_default(),
        preview: document.preview,
        excerpt: document.excerpt,
        headings: document.headings.0,
        tags: document.tags.0,
        body: document.body,
        content_hash: document.content_hash,
        updated_at: chrono_timestamp(document.updated_at)?,
    })
}

fn jiff_timestamp(value: DateTime<Utc>) -> anyhow::Result<Timestamp> {
    value
        .to_rfc3339()
        .parse()
        .map_err(|error| anyhow!("转换知识文档时间戳失败: {error}"))
}

fn chrono_timestamp(value: Timestamp) -> anyhow::Result<DateTime<Utc>> {
    value
        .to_string()
        .parse()
        .map_err(|error| anyhow!("转换 Toasty 时间戳失败: {error}"))
}
