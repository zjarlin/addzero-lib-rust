use std::{collections::BTreeMap, sync::Arc};

use addzero_persistence::PersistenceContext;
use anyhow::{Result, anyhow};
use chrono::Utc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    SecretCipher,
    pg_repo::PgRepo,
    types::{
        AiModelProvider, AiModelProviderUpsert, AiPromptButton, AiPromptButtonUpsert,
        AiProviderKind, Asset, AssetEdge, AssetEdgeUpsert, AssetGraph, AssetKind,
        AssetProviderSecret, AssetUpsert,
    },
};

#[derive(Default)]
struct MemoryStore {
    assets: BTreeMap<Uuid, Asset>,
    edges: BTreeMap<(Uuid, Uuid, String), AssetEdge>,
    providers: BTreeMap<String, (AiModelProvider, Option<String>)>,
    prompts: BTreeMap<Uuid, AiPromptButton>,
}

#[derive(Clone)]
pub struct AssetService {
    pg: Option<PgRepo>,
    memory: Arc<Mutex<MemoryStore>>,
    cipher: Option<SecretCipher>,
}

impl AssetService {
    pub fn memory_only(cipher: Option<SecretCipher>) -> Self {
        Self {
            pg: None,
            memory: Arc::new(Mutex::new(MemoryStore::default())),
            cipher,
        }
    }

    pub async fn connect(database_url: &str, master_key: Option<&str>) -> Result<Self> {
        let persistence = PersistenceContext::connect_with_url(database_url).await?;
        Ok(Self::from_persistence(&persistence, master_key))
    }

    pub async fn try_attach(database_url: Option<&str>, master_key: Option<&str>) -> Self {
        let cipher = secret_cipher(master_key);
        let Some(url) = database_url.filter(|url| !url.trim().is_empty()) else {
            return Self::memory_only(cipher);
        };
        match Self::connect(url, master_key).await {
            Ok(service) => service,
            Err(err) => {
                log::warn!("asset postgres connect failed, falling back to memory: {err:?}");
                Self::memory_only(cipher)
            }
        }
    }

    pub fn from_persistence(persistence: &PersistenceContext, master_key: Option<&str>) -> Self {
        Self {
            pg: Some(PgRepo::new(persistence.db().clone())),
            memory: Arc::new(Mutex::new(MemoryStore::default())),
            cipher: secret_cipher(master_key),
        }
    }

    pub fn is_pg_online(&self) -> bool {
        self.pg.is_some()
    }

    pub async fn list_assets(&self, kind: Option<AssetKind>) -> Result<Vec<Asset>> {
        if let Some(pg) = &self.pg {
            return pg.list_assets(kind).await;
        }
        let store = self.memory.lock().await;
        Ok(store
            .assets
            .values()
            .filter(|asset| kind.is_none_or(|expected| asset.kind == expected))
            .cloned()
            .collect())
    }

    pub async fn upsert_asset(&self, input: AssetUpsert) -> Result<Asset> {
        if let Some(pg) = &self.pg {
            return pg.upsert_asset(&input).await;
        }
        let id = input.id.unwrap_or_else(Uuid::new_v4);
        let content_hash = input.compute_hash();
        let now = Utc::now();
        let mut store = self.memory.lock().await;
        let created_at = store
            .assets
            .get(&id)
            .map(|asset| asset.created_at)
            .unwrap_or(now);
        let asset = Asset {
            id,
            kind: input.kind,
            title: input.title,
            body: input.body,
            tags: input.tags,
            status: input.status,
            metadata: input.metadata,
            content_hash,
            created_at,
            updated_at: now,
        };
        store.assets.insert(id, asset.clone());
        Ok(asset)
    }

    pub async fn delete_asset(&self, id: Uuid) -> Result<()> {
        if let Some(pg) = &self.pg {
            return pg.delete_asset(id).await;
        }
        let mut store = self.memory.lock().await;
        store.assets.remove(&id);
        store
            .edges
            .retain(|(source, target, _), _| *source != id && *target != id);
        Ok(())
    }

    pub async fn graph(&self) -> Result<AssetGraph> {
        if let Some(pg) = &self.pg {
            return pg.graph().await;
        }
        let store = self.memory.lock().await;
        Ok(AssetGraph {
            assets: store.assets.values().cloned().collect(),
            edges: store.edges.values().cloned().collect(),
        })
    }

    pub async fn upsert_edge(&self, input: AssetEdgeUpsert) -> Result<AssetEdge> {
        if let Some(pg) = &self.pg {
            return pg.upsert_edge(&input).await;
        }
        let now = Utc::now();
        let mut store = self.memory.lock().await;
        let key = (
            input.source_asset_id,
            input.target_asset_id,
            input.relation.clone(),
        );
        let created_at = store
            .edges
            .get(&key)
            .map(|edge| edge.created_at)
            .unwrap_or(now);
        let edge = AssetEdge {
            id: store
                .edges
                .get(&key)
                .map(|edge| edge.id)
                .unwrap_or_else(Uuid::new_v4),
            source_asset_id: input.source_asset_id,
            target_asset_id: input.target_asset_id,
            relation: input.relation,
            confidence: input.confidence,
            metadata: input.metadata,
            created_at,
            updated_at: now,
        };
        store.edges.insert(key, edge.clone());
        Ok(edge)
    }

    pub async fn list_providers(&self) -> Result<Vec<AiModelProvider>> {
        if let Some(pg) = &self.pg {
            return pg.list_providers().await;
        }
        let store = self.memory.lock().await;
        Ok(store
            .providers
            .values()
            .map(|(provider, _)| provider.clone())
            .collect())
    }

    pub async fn upsert_provider(&self, input: AiModelProviderUpsert) -> Result<AiModelProvider> {
        let encrypted = match input
            .api_key
            .as_deref()
            .filter(|key| !key.trim().is_empty())
        {
            Some(api_key) => {
                let cipher = self.cipher.as_ref().ok_or_else(|| {
                    anyhow!("ADDZERO_SECRET_MASTER_KEY is required to save API keys")
                })?;
                Some(cipher.encrypt(api_key)?)
            }
            None => None,
        };
        if let Some(pg) = &self.pg {
            let preserved = match encrypted {
                Some(encrypted) => Some(encrypted),
                None => pg.stored_provider_secret(input.provider).await?,
            };
            return pg.upsert_provider(&input, preserved).await;
        }
        let mut store = self.memory.lock().await;
        let now = Utc::now();
        let previous_secret = store
            .providers
            .get(input.provider.as_str())
            .and_then(|(_, secret)| secret.clone());
        let secret = encrypted.map(|value| value.ciphertext).or(previous_secret);
        let provider = AiModelProvider {
            provider: input.provider,
            default_model: input.default_model,
            enabled: input.enabled,
            key_id: self
                .cipher
                .as_ref()
                .map(|cipher| cipher.key_id().to_string())
                .unwrap_or_else(|| "default".to_string()),
            api_key_configured: secret.is_some(),
            updated_at: now,
        };
        store.providers.insert(
            provider.provider.as_str().to_string(),
            (provider.clone(), secret),
        );
        Ok(provider)
    }

    pub async fn provider_secret(
        &self,
        provider: AiProviderKind,
    ) -> Result<Option<AssetProviderSecret>> {
        if let Some(pg) = &self.pg {
            let Some((record, encrypted)) = pg.provider_secret(provider).await? else {
                return Ok(None);
            };
            let cipher = self.cipher.as_ref().ok_or_else(|| {
                anyhow!("ADDZERO_SECRET_MASTER_KEY is required to decrypt API keys")
            })?;
            return Ok(Some(AssetProviderSecret {
                provider: record.provider,
                default_model: record.default_model,
                api_key: cipher.decrypt(&encrypted)?,
            }));
        }
        let store = self.memory.lock().await;
        let Some((record, Some(ciphertext))) = store.providers.get(provider.as_str()) else {
            return Ok(None);
        };
        let cipher = self
            .cipher
            .as_ref()
            .ok_or_else(|| anyhow!("ADDZERO_SECRET_MASTER_KEY is required to decrypt API keys"))?;
        Ok(Some(AssetProviderSecret {
            provider: record.provider,
            default_model: record.default_model.clone(),
            api_key: cipher.decrypt(&crate::EncryptedSecret {
                key_id: record.key_id.clone(),
                ciphertext: ciphertext.clone(),
            })?,
        }))
    }

    pub async fn list_prompt_buttons(&self) -> Result<Vec<AiPromptButton>> {
        if let Some(pg) = &self.pg {
            return pg.list_prompt_buttons().await;
        }
        let store = self.memory.lock().await;
        Ok(store.prompts.values().cloned().collect())
    }

    pub async fn upsert_prompt_button(
        &self,
        input: AiPromptButtonUpsert,
    ) -> Result<AiPromptButton> {
        if let Some(pg) = &self.pg {
            return pg.upsert_prompt_button(&input).await;
        }
        let id = input.id.unwrap_or_else(Uuid::new_v4);
        let prompt = AiPromptButton {
            id,
            label: input.label,
            target_kind: input.target_kind,
            prompt_template: input.prompt_template,
            provider: input.provider,
            model: input.model,
            enabled: input.enabled,
            updated_at: Utc::now(),
        };
        self.memory.lock().await.prompts.insert(id, prompt.clone());
        Ok(prompt)
    }

    pub async fn delete_prompt_button(&self, id: Uuid) -> Result<()> {
        if let Some(pg) = &self.pg {
            return pg.delete_prompt_button(id).await;
        }
        self.memory.lock().await.prompts.remove(&id);
        Ok(())
    }
}

fn secret_cipher(master_key: Option<&str>) -> Option<SecretCipher> {
    master_key
        .filter(|key| !key.trim().is_empty())
        .and_then(|key| match SecretCipher::from_master_key(key) {
            Ok(cipher) => Some(cipher),
            Err(err) => {
                log::warn!("AI secret cipher disabled: {err:?}");
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AiModelProviderUpsert, AiProviderKind, AssetKind};

    #[tokio::test]
    async fn asset_service_should_crud_asset_and_edge_in_memory() {
        let service = AssetService::memory_only(None);
        let source = service
            .upsert_asset(AssetUpsert {
                id: None,
                kind: AssetKind::Capture,
                title: "raw".into(),
                body: "ownership idea".into(),
                tags: vec!["rust".into()],
                status: "active".into(),
                metadata: serde_json::json!({}),
            })
            .await
            .unwrap();
        let target = service
            .upsert_asset(AssetUpsert {
                id: None,
                kind: AssetKind::Note,
                title: "Ownership".into(),
                body: "Borrowing summary".into(),
                tags: vec!["rust".into()],
                status: "active".into(),
                metadata: serde_json::json!({}),
            })
            .await
            .unwrap();
        service
            .upsert_edge(AssetEdgeUpsert {
                source_asset_id: source.id,
                target_asset_id: target.id,
                relation: "summarizes".into(),
                confidence: 0.9,
                metadata: serde_json::json!({}),
            })
            .await
            .unwrap();
        let graph = service.graph().await.unwrap();
        assert_eq!(graph.assets.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }

    #[tokio::test]
    async fn provider_secret_should_roundtrip_encrypted_key_in_memory() {
        let cipher = SecretCipher::from_master_key("local-dev-master-key").unwrap();
        let service = AssetService::memory_only(Some(cipher));
        let provider = service
            .upsert_provider(AiModelProviderUpsert {
                provider: AiProviderKind::Anthropic,
                default_model: "claude-test".into(),
                enabled: true,
                api_key: Some("sk-ant-test".into()),
            })
            .await
            .unwrap();
        assert!(provider.api_key_configured);
        let secret = service
            .provider_secret(AiProviderKind::Anthropic)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(secret.api_key, "sk-ant-test");
        assert_eq!(secret.default_model, "claude-test");
    }
}
