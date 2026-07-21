use anyhow::{Context, Result, anyhow};
use az_persistence::context::PersistenceDb;
use chrono::{DateTime, Utc};
use jiff::Timestamp;
use toasty::stmt::{List, Query};
use uuid::Uuid;

use crate::{
    models::{
        ai_model_provider::AiModelProviderRecord, ai_prompt_button::AiPromptButtonRecord,
        asset::AssetRecord, asset_edge::AssetEdgeRecord,
    },
    secret::EncryptedSecret,
    types::{
        AiModelProvider, AiModelProviderUpsert, AiPromptButton, AiPromptButtonUpsert,
        AiProviderKind, Asset, AssetEdge, AssetEdgeUpsert, AssetGraph, AssetKind, AssetUpsert,
    },
};

#[derive(Clone)]
pub struct PgRepo {
    db: PersistenceDb,
}

impl PgRepo {
    pub fn new(db: PersistenceDb) -> Self {
        Self { db }
    }

    pub async fn list_assets(&self, kind: Option<AssetKind>) -> Result<Vec<Asset>> {
        let mut db = self.db.lock().await;
        let rows = match kind {
            Some(kind) => Query::<List<AssetRecord>>::filter(
                AssetRecord::fields().kind().eq(kind.as_str()),
            )
            .exec(&mut *db)
            .await,
            None => Query::<List<AssetRecord>>::all().exec(&mut *db).await,
        }
        .context("查询资产列表失败")?;
        let mut assets = rows
            .into_iter()
            .map(record_to_asset)
            .collect::<Result<Vec<_>>>()?;
        assets.sort_by(|left, right| {
            left.kind
                .code()
                .cmp(right.kind.code())
                .then(right.updated_at.cmp(&left.updated_at))
                .then(left.title.cmp(&right.title))
        });
        Ok(assets)
    }

    pub async fn upsert_asset(&self, input: &AssetUpsert) -> Result<Asset> {
        let id = input.id.unwrap_or_else(Uuid::new_v4);
        let now = Timestamp::now();
        let mut db = self.db.lock().await;
        let existing = Query::<List<AssetRecord>>::filter(AssetRecord::fields().id().eq(id))
            .first()
            .exec(&mut *db)
            .await
            .context("查询待保存资产失败")?;
        let record = match existing {
            Some(_) => {
                AssetRecord::filter(AssetRecord::fields().id().eq(id))
                    .update()
                    .kind(input.kind.code())
                    .title(&input.title)
                    .body(&input.body)
                    .tags(input.tags.clone())
                    .status(&input.status)
                    .metadata(input.metadata.clone())
                    .content_hash(input.compute_hash())
                    .updated_at(now)
                    .exec(&mut *db)
                    .await
                    .context("更新资产失败")?;
                Query::<List<AssetRecord>>::filter(AssetRecord::fields().id().eq(id))
                    .one()
                    .exec(&mut *db)
                    .await
                    .context("重新读取资产失败")?
            }
            None => AssetRecord::create()
                .id(id)
                .kind(input.kind.code())
                .title(&input.title)
                .body(&input.body)
                .tags(input.tags.clone())
                .status(&input.status)
                .metadata(input.metadata.clone())
                .content_hash(input.compute_hash())
                .created_at(now)
                .updated_at(now)
                .exec(&mut *db)
                .await
                .context("创建资产失败")?,
        };
        record_to_asset(record)
    }

    pub async fn delete_asset(&self, id: Uuid) -> Result<()> {
        let mut db = self.db.lock().await;
        AssetEdgeRecord::filter(
            AssetEdgeRecord::fields()
                .source_asset_id()
                .eq(id)
                .or(AssetEdgeRecord::fields().target_asset_id().eq(id)),
        )
        .delete()
        .exec(&mut *db)
        .await
        .context("删除资产关系失败")?;
        AssetRecord::filter(AssetRecord::fields().id().eq(id))
            .delete()
            .exec(&mut *db)
            .await
            .context("删除资产失败")?;
        Ok(())
    }

    pub async fn graph(&self) -> Result<AssetGraph> {
        let assets = self.list_assets(None).await?;
        let mut db = self.db.lock().await;
        let rows = Query::<List<AssetEdgeRecord>>::all()
            .exec(&mut *db)
            .await
            .context("查询资产关系失败")?;
        let mut edges = rows
            .into_iter()
            .map(record_to_edge)
            .collect::<Result<Vec<_>>>()?;
        edges.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
        Ok(AssetGraph { assets, edges })
    }

    pub async fn upsert_edge(&self, input: &AssetEdgeUpsert) -> Result<AssetEdge> {
        let filter = AssetEdgeRecord::fields()
            .source_asset_id()
            .eq(input.source_asset_id)
            .and(
                AssetEdgeRecord::fields()
                    .target_asset_id()
                    .eq(input.target_asset_id),
            )
            .and(
                AssetEdgeRecord::fields()
                    .relation()
                    .eq(&input.relation),
            );
        let now = Timestamp::now();
        let mut db = self.db.lock().await;
        let existing = Query::<List<AssetEdgeRecord>>::filter(filter.clone())
            .first()
            .exec(&mut *db)
            .await
            .context("查询待保存资产关系失败")?;
        let record = match existing {
            Some(existing) => {
                AssetEdgeRecord::filter(AssetEdgeRecord::fields().id().eq(existing.id))
                    .update()
                    .confidence(input.confidence)
                    .metadata(input.metadata.clone())
                    .updated_at(now)
                    .exec(&mut *db)
                    .await
                    .context("更新资产关系失败")?;
                Query::<List<AssetEdgeRecord>>::filter(
                    AssetEdgeRecord::fields().id().eq(existing.id),
                )
                .one()
                .exec(&mut *db)
                .await
                .context("重新读取资产关系失败")?
            }
            None => AssetEdgeRecord::create()
                .id(Uuid::new_v4())
                .source_asset_id(input.source_asset_id)
                .target_asset_id(input.target_asset_id)
                .relation(&input.relation)
                .confidence(input.confidence)
                .metadata(input.metadata.clone())
                .created_at(now)
                .updated_at(now)
                .exec(&mut *db)
                .await
                .context("创建资产关系失败")?,
        };
        record_to_edge(record)
    }

    pub async fn list_providers(&self) -> Result<Vec<AiModelProvider>> {
        let mut db = self.db.lock().await;
        let rows = Query::<List<AiModelProviderRecord>>::all()
            .exec(&mut *db)
            .await
            .context("查询 AI provider 失败")?;
        let mut providers = rows
            .into_iter()
            .map(record_to_provider)
            .collect::<Result<Vec<_>>>()?;
        providers.sort_by_key(|provider| provider.provider.code());
        Ok(providers)
    }

    pub async fn upsert_provider(
        &self,
        input: &AiModelProviderUpsert,
        encrypted: Option<EncryptedSecret>,
    ) -> Result<AiModelProvider> {
        let provider = input.provider.code();
        let now = Timestamp::now();
        let mut db = self.db.lock().await;
        let existing = Query::<List<AiModelProviderRecord>>::filter(
            AiModelProviderRecord::fields().provider().eq(provider),
        )
        .first()
        .exec(&mut *db)
        .await
        .context("查询待保存 AI provider 失败")?;
        let record = match existing {
            Some(existing) => {
                let key_id = encrypted
                    .as_ref()
                    .map(|secret| secret.key_id.clone())
                    .unwrap_or(existing.key_id);
                let ciphertext = encrypted
                    .map(|secret| secret.ciphertext)
                    .or(existing.encrypted_api_key);
                AiModelProviderRecord::filter(
                    AiModelProviderRecord::fields().provider().eq(provider),
                )
                .update()
                .base_url(input.base_url.clone())
                .default_model(&input.default_model)
                .enabled(input.enabled)
                .key_id(key_id)
                .encrypted_api_key(ciphertext.clone())
                .api_key_configured(ciphertext.is_some())
                .updated_at(now)
                .exec(&mut *db)
                .await
                .context("更新 AI provider 失败")?;
                Query::<List<AiModelProviderRecord>>::filter(
                    AiModelProviderRecord::fields().provider().eq(provider),
                )
                .one()
                .exec(&mut *db)
                .await
                .context("重新读取 AI provider 失败")?
            }
            None => {
                let secret = encrypted.map(|secret| (secret.key_id, secret.ciphertext));
                let key_id = secret
                    .as_ref()
                    .map(|(key_id, _)| key_id.clone())
                    .unwrap_or_else(|| "default".to_string());
                let ciphertext = secret.map(|(_, ciphertext)| ciphertext);
                AiModelProviderRecord::create()
                    .provider(provider)
                    .base_url(input.base_url.clone())
                    .default_model(&input.default_model)
                    .enabled(input.enabled)
                    .key_id(key_id)
                    .encrypted_api_key(ciphertext.clone())
                    .api_key_configured(ciphertext.is_some())
                    .created_at(now)
                    .updated_at(now)
                    .exec(&mut *db)
                    .await
                    .context("创建 AI provider 失败")?
            }
        };
        record_to_provider(record)
    }

    pub async fn provider_secret(
        &self,
        provider: AiProviderKind,
    ) -> Result<Option<(AiModelProvider, EncryptedSecret)>> {
        let mut db = self.db.lock().await;
        let row = Query::<List<AiModelProviderRecord>>::filter(
            AiModelProviderRecord::fields()
                .provider()
                .eq(provider.code())
                .and(AiModelProviderRecord::fields().enabled().eq(true))
                .and(
                    AiModelProviderRecord::fields()
                        .encrypted_api_key()
                        .is_some(),
                ),
        )
        .first()
        .exec(&mut *db)
        .await
        .context("查询 AI provider 密钥失败")?;
        row.map(|row| {
            let secret = EncryptedSecret {
                key_id: row.key_id.clone(),
                ciphertext: row.encrypted_api_key.clone().unwrap_or_default(),
            };
            Ok((record_to_provider(row)?, secret))
        })
        .transpose()
    }

    pub async fn stored_provider_secret(
        &self,
        provider: AiProviderKind,
    ) -> Result<Option<EncryptedSecret>> {
        let mut db = self.db.lock().await;
        let row = Query::<List<AiModelProviderRecord>>::filter(
            AiModelProviderRecord::fields()
                .provider()
                .eq(provider.code())
                .and(
                    AiModelProviderRecord::fields()
                        .encrypted_api_key()
                        .is_some(),
                ),
        )
        .first()
        .exec(&mut *db)
        .await
        .context("查询已保存 AI provider 密钥失败")?;
        Ok(row.map(|row| EncryptedSecret {
            key_id: row.key_id,
            ciphertext: row.encrypted_api_key.unwrap_or_default(),
        }))
    }

    pub async fn list_prompt_buttons(&self) -> Result<Vec<AiPromptButton>> {
        let mut db = self.db.lock().await;
        let rows = Query::<List<AiPromptButtonRecord>>::all()
            .exec(&mut *db)
            .await
            .context("查询 prompt 按钮失败")?;
        let mut buttons = rows
            .into_iter()
            .map(record_to_prompt)
            .collect::<Result<Vec<_>>>()?;
        buttons.sort_by(|left, right| {
            left.target_kind
                .code()
                .cmp(right.target_kind.code())
                .then(left.label.cmp(&right.label))
        });
        Ok(buttons)
    }

    pub async fn upsert_prompt_button(
        &self,
        input: &AiPromptButtonUpsert,
    ) -> Result<AiPromptButton> {
        let id = input.id.unwrap_or_else(Uuid::new_v4);
        let now = Timestamp::now();
        let mut db = self.db.lock().await;
        let existing = Query::<List<AiPromptButtonRecord>>::filter(
            AiPromptButtonRecord::fields().id().eq(id),
        )
        .first()
        .exec(&mut *db)
        .await
        .context("查询待保存 prompt 按钮失败")?;
        let record = match existing {
            Some(_) => {
                AiPromptButtonRecord::filter(AiPromptButtonRecord::fields().id().eq(id))
                    .update()
                    .label(&input.label)
                    .target_kind(input.target_kind.code())
                    .prompt_template(&input.prompt_template)
                    .provider(input.provider.code())
                    .model(&input.model)
                    .enabled(input.enabled)
                    .updated_at(now)
                    .exec(&mut *db)
                    .await
                    .context("更新 prompt 按钮失败")?;
                Query::<List<AiPromptButtonRecord>>::filter(
                    AiPromptButtonRecord::fields().id().eq(id),
                )
                .one()
                .exec(&mut *db)
                .await
                .context("重新读取 prompt 按钮失败")?
            }
            None => AiPromptButtonRecord::create()
                .id(id)
                .label(&input.label)
                .target_kind(input.target_kind.code())
                .prompt_template(&input.prompt_template)
                .provider(input.provider.code())
                .model(&input.model)
                .enabled(input.enabled)
                .created_at(now)
                .updated_at(now)
                .exec(&mut *db)
                .await
                .context("创建 prompt 按钮失败")?,
        };
        record_to_prompt(record)
    }

    pub async fn delete_prompt_button(&self, id: Uuid) -> Result<()> {
        let mut db = self.db.lock().await;
        AiPromptButtonRecord::filter(AiPromptButtonRecord::fields().id().eq(id))
            .delete()
            .exec(&mut *db)
            .await
            .context("删除 prompt 按钮失败")?;
        Ok(())
    }
}

fn record_to_asset(row: AssetRecord) -> Result<Asset> {
    Ok(Asset {
        id: row.id,
        kind: AssetKind::from_code_or_default(&row.kind),
        title: row.title,
        body: row.body,
        tags: row.tags.0,
        status: row.status,
        metadata: row.metadata.0,
        content_hash: row.content_hash,
        created_at: chrono_timestamp(row.created_at)?,
        updated_at: chrono_timestamp(row.updated_at)?,
    })
}

fn record_to_edge(row: AssetEdgeRecord) -> Result<AssetEdge> {
    Ok(AssetEdge {
        id: row.id,
        source_asset_id: row.source_asset_id,
        target_asset_id: row.target_asset_id,
        relation: row.relation,
        confidence: row.confidence,
        metadata: row.metadata.0,
        created_at: chrono_timestamp(row.created_at)?,
        updated_at: chrono_timestamp(row.updated_at)?,
    })
}

fn record_to_provider(row: AiModelProviderRecord) -> Result<AiModelProvider> {
    Ok(AiModelProvider {
        provider: AiProviderKind::from_code_or_default(&row.provider),
        base_url: row.base_url,
        default_model: row.default_model,
        enabled: row.enabled,
        key_id: row.key_id,
        api_key_configured: row.api_key_configured,
        updated_at: chrono_timestamp(row.updated_at)?,
    })
}

fn record_to_prompt(row: AiPromptButtonRecord) -> Result<AiPromptButton> {
    Ok(AiPromptButton {
        id: row.id,
        label: row.label,
        target_kind: AssetKind::from_code_or_default(&row.target_kind),
        prompt_template: row.prompt_template,
        provider: AiProviderKind::from_code_or_default(&row.provider),
        model: row.model,
        enabled: row.enabled,
        updated_at: chrono_timestamp(row.updated_at)?,
    })
}

fn chrono_timestamp(value: Timestamp) -> Result<DateTime<Utc>> {
    value
        .to_string()
        .parse()
        .map_err(|error| anyhow!("转换 Toasty 时间戳失败: {error}"))
}
