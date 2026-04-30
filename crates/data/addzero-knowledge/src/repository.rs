use std::collections::BTreeMap;

use chrono::Utc;
use sea_orm::{
    ActiveValue::NotSet,
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
    sea_query::{Expr, OnConflict},
};
use uuid::Uuid;

use crate::{
    entity::{knowledge_document, knowledge_source},
    types::{KnowledgeDocument, KnowledgeError, KnowledgeSourceSpec},
};

#[derive(Clone)]
pub(crate) struct KnowledgeRepository {
    db: DatabaseConnection,
}

impl KnowledgeRepository {
    pub(crate) fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub(crate) async fn list_documents(&self) -> Result<Vec<KnowledgeDocument>, KnowledgeError> {
        let sources = knowledge_source::Entity::find()
            .all(&self.db)
            .await
            .map_err(KnowledgeError::Query)?;
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
            .map_err(KnowledgeError::Query)?;

        docs.into_iter()
            .map(|doc| {
                let source = source_map.get(&doc.source_id).ok_or_else(|| {
                    KnowledgeError::Message(format!(
                        "missing knowledge source for document {}",
                        doc.source_path
                    ))
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
                    body: doc.body,
                    content_hash: doc.content_hash,
                })
            })
            .collect()
    }

    pub(crate) async fn upsert_source(
        &self,
        source: &KnowledgeSourceSpec,
    ) -> Result<Uuid, KnowledgeError> {
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
            .map_err(KnowledgeError::Query)?;

        knowledge_source::Entity::find()
            .filter(knowledge_source::Column::Slug.eq(source.slug.clone()))
            .one(&self.db)
            .await
            .map_err(KnowledgeError::Query)?
            .map(|model| model.id)
            .ok_or_else(|| {
                KnowledgeError::Message(format!("failed to load source {}", source.slug))
            })
    }

    pub(crate) async fn upsert_document(
        &self,
        source_id: Uuid,
        doc: &KnowledgeDocument,
    ) -> Result<(), KnowledgeError> {
        let now = Utc::now();
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
            body: Set(doc.body.clone()),
            content_hash: Set(doc.content_hash.clone()),
            is_active: Set(true),
            created_at: NotSet,
            updated_at: Set(now),
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
                        knowledge_document::Column::Body,
                        knowledge_document::Column::ContentHash,
                        knowledge_document::Column::IsActive,
                        knowledge_document::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(KnowledgeError::Query)?;

        Ok(())
    }

    pub(crate) async fn deactivate_missing_documents(
        &self,
        source_id: Uuid,
        active_paths: &[String],
    ) -> Result<(), KnowledgeError> {
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

        update.exec(&self.db).await.map_err(KnowledgeError::Query)?;
        Ok(())
    }
}
