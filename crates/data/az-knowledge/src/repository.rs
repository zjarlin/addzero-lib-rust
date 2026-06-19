use std::collections::BTreeMap;

use anyhow::{Context, anyhow};
use chrono::Utc;
use sea_orm::{
    ActiveValue::NotSet,
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
    sea_query::{Expr, OnConflict},
};
use uuid::Uuid;

use crate::{
    entity::{knowledge_document, knowledge_source},
    types::{KnowledgeDocument, KnowledgeSourceSpec},
};

#[derive(Clone)]
pub(crate) struct KnowledgeRepository {
    db: DatabaseConnection,
}

impl KnowledgeRepository {
    pub(crate) fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub(crate) async fn list_documents(&self) -> anyhow::Result<Vec<KnowledgeDocument>> {
        let sources = knowledge_source::Entity::find()
            .all(&self.db)
            .await
            .context("query knowledge sources")?;
        let source_map = sources
            .into_iter()
            .map(|source| (source.id, source))
            .collect::<BTreeMap<_, _>>();

        let docs = knowledge_document::Entity::find()
            .filter(knowledge_document::Column::IsActive.eq(true))
            .order_by_asc(knowledge_document::Column::SourceId)
            .order_by_asc(knowledge_document::Column::RelativePath)
            .all(&self.db)
            .await
            .context("query active knowledge documents")?;

        docs.into_iter()
            .map(|doc| {
                let source = source_map.get(&doc.source_id).ok_or_else(|| {
                    anyhow!(
                        "missing knowledge source for document {}",
                        doc.source_path
                    )
                })?;
                Ok(KnowledgeDocument {
                    source_slug: source.slug.clone(),
                    source_name: source.name.clone(),
                    source_root: source.root_path.clone(),
                    slug: doc.slug,
                    title: doc.title,
                    filename: doc.filename,
                    source_path: doc.source_path,
                    relative_path: doc.relative_path,
                    bytes: usize::try_from(doc.bytes).unwrap_or_default(),
                    section_count: usize::try_from(doc.section_count).unwrap_or_default(),
                    preview: doc.preview,
                    excerpt: doc.excerpt,
                    headings: doc.headings,
                    tags: doc.tags,
                    body: doc.body,
                    content_hash: doc.content_hash,
                    updated_at: doc.updated_at,
                })
            })
            .collect()
    }

    pub(crate) async fn upsert_source(
        &self,
        source: &KnowledgeSourceSpec,
    ) -> anyhow::Result<Uuid> {
        let now = Utc::now();
        let active = knowledge_source::ActiveModel {
            id: Set(Uuid::new_v4()),
            slug: Set(source.slug.clone()),
            name: Set(source.name.clone()),
            root_path: Set(source.root_path.display().to_string()),
            last_synced_at: Set(Some(now)),
            created_at: NotSet,
            updated_at: Set(now),
        };

        knowledge_source::Entity::insert(active)
            .on_conflict(
                OnConflict::column(knowledge_source::Column::Slug)
                    .update_columns([
                        knowledge_source::Column::Name,
                        knowledge_source::Column::RootPath,
                        knowledge_source::Column::LastSyncedAt,
                        knowledge_source::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .with_context(|| format!("upsert knowledge source `{}`", source.slug))?;

        knowledge_source::Entity::find()
            .filter(knowledge_source::Column::Slug.eq(source.slug.clone()))
            .one(&self.db)
            .await
            .with_context(|| format!("load knowledge source `{}`", source.slug))?
            .map(|model| model.id)
            .ok_or_else(|| anyhow!("failed to load source {}", source.slug))
    }

    pub(crate) async fn upsert_document(
        &self,
        source_id: Uuid,
        doc: &KnowledgeDocument,
    ) -> anyhow::Result<()> {
        let active = knowledge_document::ActiveModel {
            id: Set(Uuid::new_v4()),
            source_id: Set(source_id),
            slug: Set(doc.slug.clone()),
            title: Set(doc.title.clone()),
            filename: Set(doc.filename.clone()),
            source_path: Set(doc.source_path.clone()),
            relative_path: Set(doc.relative_path.clone()),
            bytes: Set(i64::try_from(doc.bytes).unwrap_or_default()),
            section_count: Set(i32::try_from(doc.section_count).unwrap_or_default()),
            preview: Set(doc.preview.clone()),
            excerpt: Set(doc.excerpt.clone()),
            headings: Set(doc.headings.clone()),
            tags: Set(doc.tags.clone()),
            body: Set(doc.body.clone()),
            content_hash: Set(doc.content_hash.clone()),
            is_active: Set(true),
            created_at: NotSet,
            updated_at: Set(doc.updated_at),
        };

        knowledge_document::Entity::insert(active)
            .on_conflict(
                OnConflict::column(knowledge_document::Column::SourcePath)
                    .update_columns([
                        knowledge_document::Column::SourceId,
                        knowledge_document::Column::Slug,
                        knowledge_document::Column::Title,
                        knowledge_document::Column::Filename,
                        knowledge_document::Column::RelativePath,
                        knowledge_document::Column::Bytes,
                        knowledge_document::Column::SectionCount,
                        knowledge_document::Column::Preview,
                        knowledge_document::Column::Excerpt,
                        knowledge_document::Column::Headings,
                        knowledge_document::Column::Tags,
                        knowledge_document::Column::Body,
                        knowledge_document::Column::ContentHash,
                        knowledge_document::Column::IsActive,
                        knowledge_document::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .with_context(|| format!("upsert knowledge document `{}`", doc.source_path))?;

        Ok(())
    }

    pub(crate) async fn source_by_slug(
        &self,
        slug: &str,
    ) -> anyhow::Result<Option<knowledge_source::Model>> {
        knowledge_source::Entity::find()
            .filter(knowledge_source::Column::Slug.eq(slug.to_string()))
            .one(&self.db)
            .await
            .with_context(|| format!("load knowledge source `{slug}`"))
    }

    pub(crate) async fn deactivate_missing_documents(
        &self,
        source_id: Uuid,
        active_paths: &[String],
    ) -> anyhow::Result<()> {
        let now = Utc::now();
        let mut update = knowledge_document::Entity::update_many()
            .col_expr(knowledge_document::Column::IsActive, Expr::value(false))
            .col_expr(knowledge_document::Column::UpdatedAt, Expr::value(now))
            .filter(knowledge_document::Column::SourceId.eq(source_id));

        if !active_paths.is_empty() {
            update = update.filter(
                knowledge_document::Column::SourcePath.is_not_in(active_paths.iter().cloned()),
            );
        }

        update
            .exec(&self.db)
            .await
            .context("deactivate missing knowledge documents")?;
        Ok(())
    }

    pub(crate) async fn deactivate_document_by_source_path(
        &self,
        source_path: &str,
    ) -> anyhow::Result<()> {
        let now = Utc::now();
        knowledge_document::Entity::update_many()
            .col_expr(knowledge_document::Column::IsActive, Expr::value(false))
            .col_expr(knowledge_document::Column::UpdatedAt, Expr::value(now))
            .filter(knowledge_document::Column::SourcePath.eq(source_path.to_string()))
            .exec(&self.db)
            .await
            .with_context(|| format!("deactivate knowledge document `{source_path}`"))?;
        Ok(())
    }
}
