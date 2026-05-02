use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use sea_orm::{
    ActiveValue::NotSet, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Set, sea_query::OnConflict,
};
use uuid::Uuid;

use crate::{
    EncryptedSecret,
    entity::{ai_model_provider, ai_prompt_button, asset, asset_edge},
    types::{
        AiModelProvider, AiModelProviderUpsert, AiPromptButton, AiPromptButtonUpsert,
        AiProviderKind, Asset, AssetEdge, AssetEdgeUpsert, AssetGraph, AssetKind, AssetUpsert,
    },
};

#[derive(Clone)]
pub struct PgRepo {
    db: DatabaseConnection,
}

impl PgRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn list_assets(&self, kind: Option<AssetKind>) -> Result<Vec<Asset>> {
        let mut query = asset::Entity::find();
        if let Some(kind) = kind {
            query = query.filter(asset::Column::Kind.eq(kind.as_str()));
        }
        let rows = query
            .order_by_asc(asset::Column::Kind)
            .order_by_desc(asset::Column::UpdatedAt)
            .order_by_asc(asset::Column::Title)
            .all(&self.db)
            .await
            .context("list assets")?;
        Ok(rows.into_iter().map(model_to_asset).collect())
    }

    pub async fn upsert_asset(&self, input: &AssetUpsert) -> Result<Asset> {
        let id = input.id.unwrap_or_else(Uuid::new_v4);
        let active = asset::ActiveModel {
            id: Set(id),
            kind: Set(input.kind.as_str().to_string()),
            title: Set(input.title.clone()),
            body: Set(input.body.clone()),
            tags: Set(input.tags.clone()),
            status: Set(input.status.clone()),
            metadata: Set(input.metadata.clone()),
            content_hash: Set(input.compute_hash()),
            created_at: NotSet,
            updated_at: Set(Utc::now()),
        };

        asset::Entity::insert(active)
            .on_conflict(
                OnConflict::column(asset::Column::Id)
                    .update_columns([
                        asset::Column::Kind,
                        asset::Column::Title,
                        asset::Column::Body,
                        asset::Column::Tags,
                        asset::Column::Status,
                        asset::Column::Metadata,
                        asset::Column::ContentHash,
                        asset::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .context("upsert asset")?;

        asset::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .context("reload asset after upsert")?
            .map(model_to_asset)
            .ok_or_else(|| anyhow!("asset disappeared after upsert"))
    }

    pub async fn delete_asset(&self, id: Uuid) -> Result<()> {
        asset::Entity::delete_many()
            .filter(asset::Column::Id.eq(id))
            .exec(&self.db)
            .await
            .context("delete asset")?;
        Ok(())
    }

    pub async fn graph(&self) -> Result<AssetGraph> {
        let assets = self.list_assets(None).await?;
        let edges = asset_edge::Entity::find()
            .order_by_desc(asset_edge::Column::UpdatedAt)
            .all(&self.db)
            .await
            .context("list asset edges")?;
        Ok(AssetGraph {
            assets,
            edges: edges.into_iter().map(model_to_edge).collect(),
        })
    }

    pub async fn upsert_edge(&self, input: &AssetEdgeUpsert) -> Result<AssetEdge> {
        let active = asset_edge::ActiveModel {
            id: Set(Uuid::new_v4()),
            source_asset_id: Set(input.source_asset_id),
            target_asset_id: Set(input.target_asset_id),
            relation: Set(input.relation.clone()),
            confidence: Set(input.confidence),
            metadata: Set(input.metadata.clone()),
            created_at: NotSet,
            updated_at: Set(Utc::now()),
        };

        asset_edge::Entity::insert(active)
            .on_conflict(
                OnConflict::columns([
                    asset_edge::Column::SourceAssetId,
                    asset_edge::Column::TargetAssetId,
                    asset_edge::Column::Relation,
                ])
                .update_columns([
                    asset_edge::Column::Confidence,
                    asset_edge::Column::Metadata,
                    asset_edge::Column::UpdatedAt,
                ])
                .to_owned(),
            )
            .exec(&self.db)
            .await
            .context("upsert asset edge")?;

        asset_edge::Entity::find()
            .filter(
                Condition::all()
                    .add(asset_edge::Column::SourceAssetId.eq(input.source_asset_id))
                    .add(asset_edge::Column::TargetAssetId.eq(input.target_asset_id))
                    .add(asset_edge::Column::Relation.eq(input.relation.clone())),
            )
            .one(&self.db)
            .await
            .context("reload asset edge after upsert")?
            .map(model_to_edge)
            .ok_or_else(|| anyhow!("asset edge disappeared after upsert"))
    }

    pub async fn list_providers(&self) -> Result<Vec<AiModelProvider>> {
        let rows = ai_model_provider::Entity::find()
            .order_by_asc(ai_model_provider::Column::Provider)
            .all(&self.db)
            .await
            .context("list ai model providers")?;
        Ok(rows.into_iter().map(model_to_provider).collect())
    }

    pub async fn upsert_provider(
        &self,
        input: &AiModelProviderUpsert,
        encrypted: Option<EncryptedSecret>,
    ) -> Result<AiModelProvider> {
        let (key_id, ciphertext, api_key_configured) = match encrypted {
            Some(secret) => (secret.key_id, Some(secret.ciphertext), true),
            None => ("default".to_string(), None, false),
        };
        let active = ai_model_provider::ActiveModel {
            provider: Set(input.provider.as_str().to_string()),
            default_model: Set(input.default_model.clone()),
            enabled: Set(input.enabled),
            key_id: Set(key_id),
            encrypted_api_key: Set(ciphertext),
            api_key_configured: Set(api_key_configured),
            created_at: NotSet,
            updated_at: Set(Utc::now()),
        };

        ai_model_provider::Entity::insert(active)
            .on_conflict(
                OnConflict::column(ai_model_provider::Column::Provider)
                    .update_columns([
                        ai_model_provider::Column::DefaultModel,
                        ai_model_provider::Column::Enabled,
                        ai_model_provider::Column::KeyId,
                        ai_model_provider::Column::EncryptedApiKey,
                        ai_model_provider::Column::ApiKeyConfigured,
                        ai_model_provider::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .context("upsert ai model provider")?;

        ai_model_provider::Entity::find_by_id(input.provider.as_str().to_string())
            .one(&self.db)
            .await
            .context("reload provider after upsert")?
            .map(model_to_provider)
            .ok_or_else(|| anyhow!("provider disappeared after upsert"))
    }

    pub async fn provider_secret(
        &self,
        provider: AiProviderKind,
    ) -> Result<Option<(AiModelProvider, EncryptedSecret)>> {
        let row = ai_model_provider::Entity::find_by_id(provider.as_str().to_string())
            .filter(ai_model_provider::Column::Enabled.eq(true))
            .filter(ai_model_provider::Column::EncryptedApiKey.is_not_null())
            .one(&self.db)
            .await
            .context("get provider secret")?;
        Ok(row.map(|row| {
            let secret = EncryptedSecret {
                key_id: row.key_id.clone(),
                ciphertext: row.encrypted_api_key.clone().unwrap_or_default(),
            };
            (model_to_provider(row), secret)
        }))
    }

    pub async fn stored_provider_secret(
        &self,
        provider: AiProviderKind,
    ) -> Result<Option<EncryptedSecret>> {
        let row = ai_model_provider::Entity::find_by_id(provider.as_str().to_string())
            .filter(ai_model_provider::Column::EncryptedApiKey.is_not_null())
            .one(&self.db)
            .await
            .context("get stored provider secret")?;
        Ok(row.map(|row| EncryptedSecret {
            key_id: row.key_id,
            ciphertext: row.encrypted_api_key.unwrap_or_default(),
        }))
    }

    pub async fn list_prompt_buttons(&self) -> Result<Vec<AiPromptButton>> {
        let rows = ai_prompt_button::Entity::find()
            .order_by_asc(ai_prompt_button::Column::TargetKind)
            .order_by_asc(ai_prompt_button::Column::Label)
            .all(&self.db)
            .await
            .context("list prompt buttons")?;
        Ok(rows.into_iter().map(model_to_prompt).collect())
    }

    pub async fn upsert_prompt_button(
        &self,
        input: &AiPromptButtonUpsert,
    ) -> Result<AiPromptButton> {
        let id = input.id.unwrap_or_else(Uuid::new_v4);
        let active = ai_prompt_button::ActiveModel {
            id: Set(id),
            label: Set(input.label.clone()),
            target_kind: Set(input.target_kind.as_str().to_string()),
            prompt_template: Set(input.prompt_template.clone()),
            provider: Set(input.provider.as_str().to_string()),
            model: Set(input.model.clone()),
            enabled: Set(input.enabled),
            created_at: NotSet,
            updated_at: Set(Utc::now()),
        };

        ai_prompt_button::Entity::insert(active)
            .on_conflict(
                OnConflict::column(ai_prompt_button::Column::Id)
                    .update_columns([
                        ai_prompt_button::Column::Label,
                        ai_prompt_button::Column::TargetKind,
                        ai_prompt_button::Column::PromptTemplate,
                        ai_prompt_button::Column::Provider,
                        ai_prompt_button::Column::Model,
                        ai_prompt_button::Column::Enabled,
                        ai_prompt_button::Column::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .context("upsert prompt button")?;

        ai_prompt_button::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .context("reload prompt button after upsert")?
            .map(model_to_prompt)
            .ok_or_else(|| anyhow!("prompt button disappeared after upsert"))
    }

    pub async fn delete_prompt_button(&self, id: Uuid) -> Result<()> {
        ai_prompt_button::Entity::delete_many()
            .filter(ai_prompt_button::Column::Id.eq(id))
            .exec(&self.db)
            .await
            .context("delete prompt button")?;
        Ok(())
    }
}

fn model_to_asset(row: asset::Model) -> Asset {
    Asset {
        id: row.id,
        kind: AssetKind::from_db_value(&row.kind),
        title: row.title,
        body: row.body,
        tags: row.tags,
        status: row.status,
        metadata: row.metadata,
        content_hash: row.content_hash,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn model_to_edge(row: asset_edge::Model) -> AssetEdge {
    AssetEdge {
        id: row.id,
        source_asset_id: row.source_asset_id,
        target_asset_id: row.target_asset_id,
        relation: row.relation,
        confidence: row.confidence,
        metadata: row.metadata,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn model_to_provider(row: ai_model_provider::Model) -> AiModelProvider {
    AiModelProvider {
        provider: AiProviderKind::from_db_value(&row.provider),
        default_model: row.default_model,
        enabled: row.enabled,
        key_id: row.key_id,
        api_key_configured: row.api_key_configured,
        updated_at: row.updated_at,
    }
}

fn model_to_prompt(row: ai_prompt_button::Model) -> AiPromptButton {
    AiPromptButton {
        id: row.id,
        label: row.label,
        target_kind: AssetKind::from_db_value(&row.target_kind),
        prompt_template: row.prompt_template,
        provider: AiProviderKind::from_db_value(&row.provider),
        model: row.model,
        enabled: row.enabled,
        updated_at: row.updated_at,
    }
}
