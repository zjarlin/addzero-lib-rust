use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, Utc};
use sqlx::{
    Row,
    postgres::{PgPool, PgPoolOptions},
};
use tokio::sync::Mutex;
use uuid::Uuid;

use addzero_agent_runtime_contract::{
    AgentArtifact, AgentArtifactChannel, AgentHeartbeat, AgentNode, AgentNodeStatus,
    AgentRuntimeOverview, ConflictResolution, PairingCreateResponse, PairingExchangeRequest,
    PairingExchangeResponse, PairingRequest, PairingSessionSummary, PairingStatus,
    ResolveConflictRequest, SkillConflict, SkillSnapshot, SkillSyncRequest, SkillSyncResponse,
};
use addzero_skills::{Skill, SkillService, SkillUpsert};

const SCHEMA_SQL: &str = include_str!("migrations/0001_agent_runtime.sql");
const OFFLINE_AFTER_SECS: i64 = 180;

#[derive(Clone)]
pub struct AgentRuntimeService {
    base_url: String,
    pg: Option<PgRepo>,
    store: std::sync::Arc<Mutex<MemoryRuntimeStore>>,
}

#[derive(Clone)]
struct PairingRecord {
    id: Uuid,
    channel: AgentArtifactChannel,
    device_name: String,
    platform: String,
    agent_version: String,
    status: PairingStatus,
    poll_token_hash: String,
    expires_at: DateTime<Utc>,
    approved_at: Option<DateTime<Utc>>,
    exchanged_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
struct NodeRecord {
    id: Uuid,
    display_name: String,
    platform: String,
    channel: AgentArtifactChannel,
    agent_version: String,
    status: AgentNodeStatus,
    token_hash: String,
    paired_at: DateTime<Utc>,
    last_seen_at: Option<DateTime<Utc>>,
    last_sync_at: Option<DateTime<Utc>>,
    last_uploaded_count: usize,
    last_downloaded_count: usize,
    last_conflict_count: usize,
}

#[derive(Clone)]
struct ConflictRecord {
    id: Uuid,
    node_id: Uuid,
    skill_name: String,
    server_hash: String,
    agent_hash: String,
    server_updated_at: Option<DateTime<Utc>>,
    agent_updated_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    resolved_at: Option<DateTime<Utc>>,
    resolution: Option<ConflictResolution>,
    agent_keywords: Vec<String>,
    agent_description: String,
    agent_body: String,
}

#[derive(Clone)]
struct SkillStateRecord {
    node_id: Uuid,
    skill_name: String,
    last_synced_hash: String,
    last_synced_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
struct MemoryRuntimeStore {
    artifacts: Vec<AgentArtifact>,
    pairings: BTreeMap<Uuid, PairingRecord>,
    nodes: BTreeMap<Uuid, NodeRecord>,
    conflicts: BTreeMap<Uuid, ConflictRecord>,
    skill_states: BTreeMap<(Uuid, String), SkillStateRecord>,
}

#[derive(Clone)]
struct PgRepo {
    pool: PgPool,
}

impl AgentRuntimeService {
    pub async fn try_attach(database_url: Option<&str>, base_url: String) -> Self {
        let mut store = MemoryRuntimeStore {
            artifacts: default_artifacts(),
            ..MemoryRuntimeStore::default()
        };

        let pg = match database_url.filter(|value| !value.trim().is_empty()) {
            Some(database_url) => match PgRepo::connect(database_url).await {
                Ok(pg) => match pg.ensure_schema().await {
                    Ok(()) => {
                        if let Ok(loaded) = pg.load_store().await {
                            store = loaded.with_seeded_artifacts();
                        }
                        if let Err(err) = pg.persist_store(&store).await {
                            log::warn!("failed to persist initial runtime store: {err:?}");
                        }
                        Some(pg)
                    }
                    Err(err) => {
                        log::warn!("agent runtime schema bootstrap failed: {err:?}");
                        None
                    }
                },
                Err(err) => {
                    log::warn!("agent runtime PG connect failed: {err:?}");
                    None
                }
            },
            None => None,
        };

        Self {
            base_url,
            pg,
            store: std::sync::Arc::new(Mutex::new(store)),
        }
    }

    pub fn pg_online(&self) -> bool {
        self.pg.is_some()
    }

    pub async fn overview(&self, fs_root: String, pg_online: bool) -> Result<AgentRuntimeOverview> {
        let mut store = self.store.lock().await;
        normalize_pairings(&mut store.pairings);
        let now = Utc::now();
        let active_node = store
            .nodes
            .values()
            .filter(|node| node.status != AgentNodeStatus::Revoked)
            .max_by_key(|node| node.paired_at)
            .map(|node| node.as_public(now));
        let mut pairing_sessions = store.pairings.values().cloned().collect::<Vec<_>>();
        pairing_sessions.sort_by_key(|pairing| pairing.expires_at);
        pairing_sessions.reverse();

        let mut conflicts = store
            .conflicts
            .values()
            .filter(|conflict| conflict.resolved_at.is_none())
            .cloned()
            .collect::<Vec<_>>();
        conflicts.sort_by_key(|conflict| conflict.created_at);
        conflicts.reverse();

        Ok(AgentRuntimeOverview {
            artifacts: store.artifacts.clone(),
            active_node,
            pairing_sessions: pairing_sessions
                .into_iter()
                .take(6)
                .map(|pairing| pairing.as_summary(&self.base_url))
                .collect(),
            conflicts: conflicts
                .into_iter()
                .take(12)
                .map(|conflict| conflict.as_public())
                .collect(),
            fs_root,
            pg_online,
        })
    }

    pub async fn create_pairing(&self, input: PairingRequest) -> Result<PairingCreateResponse> {
        let mut store = self.store.lock().await;
        normalize_pairings(&mut store.pairings);
        let now = Utc::now();
        let id = Uuid::new_v4();
        let poll_token = format!("{}.{}", Uuid::new_v4(), Uuid::new_v4());
        let record = PairingRecord {
            id,
            channel: input.channel,
            device_name: input.device_name.trim().to_string(),
            platform: input.platform.trim().to_string(),
            agent_version: input.agent_version.trim().to_string(),
            status: PairingStatus::Pending,
            poll_token_hash: hash_secret(&poll_token),
            expires_at: now + chrono::Duration::minutes(10),
            approved_at: None,
            exchanged_at: None,
        };
        store.pairings.insert(id, record.clone());
        self.persist_locked(&store).await?;

        Ok(PairingCreateResponse {
            session: record.as_summary(&self.base_url),
            poll_token,
        })
    }

    pub async fn get_pairing(
        &self,
        id: Uuid,
        poll_token: Option<&str>,
    ) -> Result<PairingSessionSummary> {
        let mut store = self.store.lock().await;
        normalize_pairings(&mut store.pairings);
        let pairing = store
            .pairings
            .get(&id)
            .cloned()
            .context("pairing session not found")?;
        if let Some(poll_token) = poll_token {
            if pairing.poll_token_hash != hash_secret(poll_token) {
                bail!("invalid pairing poll token");
            }
        }
        Ok(pairing.as_summary(&self.base_url))
    }

    pub async fn approve_pairing(&self, id: Uuid) -> Result<PairingSessionSummary> {
        let mut store = self.store.lock().await;
        normalize_pairings(&mut store.pairings);
        let pairing = store
            .pairings
            .get_mut(&id)
            .context("pairing session not found")?;
        if pairing.status != PairingStatus::Pending {
            return Ok(pairing.clone().as_summary(&self.base_url));
        }
        pairing.status = PairingStatus::Approved;
        pairing.approved_at = Some(Utc::now());
        let summary = pairing.clone().as_summary(&self.base_url);
        self.persist_locked(&store).await?;
        Ok(summary)
    }

    pub async fn exchange_pairing(
        &self,
        id: Uuid,
        input: PairingExchangeRequest,
    ) -> Result<PairingExchangeResponse> {
        let mut store = self.store.lock().await;
        normalize_pairings(&mut store.pairings);
        let now = Utc::now();
        let (device_name, platform, channel, agent_version) = {
            let pairing = store
                .pairings
                .get_mut(&id)
                .context("pairing session not found")?;
            if pairing.poll_token_hash != hash_secret(&input.poll_token) {
                bail!("invalid pairing poll token");
            }
            if pairing.status == PairingStatus::Expired {
                bail!("pairing session expired");
            }
            if pairing.status == PairingStatus::Revoked {
                bail!("pairing session revoked");
            }
            if pairing.status == PairingStatus::Exchanged {
                bail!("pairing session already exchanged");
            }
            if pairing.status != PairingStatus::Approved {
                bail!("pairing session is not approved");
            }
            pairing.status = PairingStatus::Exchanged;
            pairing.exchanged_at = Some(now);
            (
                pairing.device_name.clone(),
                pairing.platform.clone(),
                pairing.channel,
                pairing.agent_version.clone(),
            )
        };

        for node in store.nodes.values_mut() {
            if node.status != AgentNodeStatus::Revoked {
                node.status = AgentNodeStatus::Revoked;
            }
        }

        let node_token = format!("node.{}.{}", Uuid::new_v4(), Uuid::new_v4());
        let node = NodeRecord {
            id: Uuid::new_v4(),
            display_name: device_name,
            platform,
            channel,
            agent_version,
            status: AgentNodeStatus::Pending,
            token_hash: hash_secret(&node_token),
            paired_at: now,
            last_seen_at: None,
            last_sync_at: None,
            last_uploaded_count: 0,
            last_downloaded_count: 0,
            last_conflict_count: 0,
        };
        store.nodes.insert(node.id, node.clone());
        self.persist_locked(&store).await?;

        Ok(PairingExchangeResponse {
            node: node.as_public(now),
            node_token,
        })
    }

    pub async fn heartbeat(&self, input: AgentHeartbeat) -> Result<AgentNode> {
        let mut store = self.store.lock().await;
        let node = find_node_mut(&mut store.nodes, &input.node_token)?;
        node.platform = input.platform.trim().to_string();
        node.agent_version = input.agent_version.trim().to_string();
        node.status = AgentNodeStatus::Online;
        node.last_seen_at = Some(Utc::now());
        let public = node.as_public(Utc::now());
        self.persist_locked(&store).await?;
        Ok(public)
    }

    pub async fn sync_skills(
        &self,
        input: SkillSyncRequest,
        skills_service: &SkillService,
    ) -> Result<SkillSyncResponse> {
        let mut store = self.store.lock().await;
        let now = Utc::now();
        let (node_id, node_public) = {
            let node = find_node_mut(&mut store.nodes, &input.node_token)?;
            node.last_seen_at = Some(now);
            node.status = AgentNodeStatus::Online;
            (node.id, node.as_public(now))
        };

        let server_skills = skills_service
            .list()
            .await
            .context("list canonical skills")?;
        let server_by_name = server_skills
            .into_iter()
            .map(|skill| (skill.name.clone(), skill))
            .collect::<BTreeMap<_, _>>();
        let local_by_name = input
            .skills
            .into_iter()
            .map(|skill| (skill.name.clone(), skill))
            .collect::<BTreeMap<_, _>>();
        let state_by_name = store
            .skill_states
            .iter()
            .filter(|((current_node_id, _), _)| *current_node_id == node_id)
            .map(|((_, skill_name), state)| (skill_name.clone(), state.clone()))
            .collect::<BTreeMap<_, _>>();

        let mut all_names = BTreeSet::new();
        all_names.extend(server_by_name.keys().cloned());
        all_names.extend(local_by_name.keys().cloned());

        let mut uploaded_names = Vec::new();
        let mut download_skills = Vec::new();
        let mut conflicts = Vec::new();

        for name in all_names {
            let server_skill = server_by_name.get(&name);
            let local_skill = local_by_name.get(&name);
            let sync_state = state_by_name.get(&name);

            match (server_skill, local_skill, sync_state) {
                (Some(server_skill), Some(local_skill), _)
                    if server_skill.content_hash == local_skill.content_hash =>
                {
                    upsert_state(
                        &mut store.skill_states,
                        node_id,
                        &name,
                        server_skill.content_hash.clone(),
                        now,
                    );
                }
                (None, Some(local_skill), _) => {
                    let upsert = snapshot_to_upsert(local_skill);
                    let saved = skills_service
                        .upsert(upsert)
                        .await
                        .with_context(|| format!("upsert local skill {}", local_skill.name))?;
                    upsert_state(
                        &mut store.skill_states,
                        node_id,
                        &saved.name,
                        saved.content_hash.clone(),
                        now,
                    );
                    uploaded_names.push(saved.name);
                }
                (Some(server_skill), None, None) => {
                    download_skills.push(skill_to_snapshot(server_skill));
                }
                (Some(server_skill), None, Some(sync_state))
                    if sync_state.last_synced_hash != server_skill.content_hash =>
                {
                    download_skills.push(skill_to_snapshot(server_skill));
                }
                (Some(server_skill), Some(local_skill), Some(sync_state))
                    if sync_state.last_synced_hash == server_skill.content_hash
                        && sync_state.last_synced_hash != local_skill.content_hash =>
                {
                    let upsert = snapshot_to_upsert(local_skill);
                    let saved = skills_service
                        .upsert(upsert)
                        .await
                        .with_context(|| format!("push agent skill {}", local_skill.name))?;
                    upsert_state(
                        &mut store.skill_states,
                        node_id,
                        &saved.name,
                        saved.content_hash.clone(),
                        now,
                    );
                    uploaded_names.push(saved.name);
                }
                (Some(server_skill), Some(_local_skill), Some(sync_state))
                    if sync_state.last_synced_hash == server_skill.content_hash =>
                {
                    download_skills.push(skill_to_snapshot(server_skill));
                }
                (Some(server_skill), Some(local_skill), Some(sync_state))
                    if sync_state.last_synced_hash == local_skill.content_hash =>
                {
                    download_skills.push(skill_to_snapshot(server_skill));
                }
                (Some(server_skill), Some(local_skill), None) => {
                    conflicts.push(upsert_conflict(
                        &mut store.conflicts,
                        node_id,
                        server_skill,
                        local_skill,
                    ));
                }
                (Some(server_skill), Some(local_skill), Some(sync_state))
                    if sync_state.last_synced_hash != server_skill.content_hash
                        && sync_state.last_synced_hash != local_skill.content_hash =>
                {
                    conflicts.push(upsert_conflict(
                        &mut store.conflicts,
                        node_id,
                        server_skill,
                        local_skill,
                    ));
                }
                _ => {}
            }
        }

        if let Some(node) = store.nodes.get_mut(&node_id) {
            node.last_sync_at = Some(now);
            node.last_uploaded_count = uploaded_names.len();
            node.last_downloaded_count = download_skills.len();
            node.last_conflict_count = conflicts.len();
        }

        self.persist_locked(&store).await?;

        Ok(SkillSyncResponse {
            node: store
                .nodes
                .get(&node_id)
                .map(|node| node.as_public(now))
                .unwrap_or(node_public),
            uploaded_names,
            download_skills,
            conflicts,
            synced_at: now,
        })
    }

    pub async fn resolve_conflict(
        &self,
        id: Uuid,
        input: ResolveConflictRequest,
        skills_service: &SkillService,
    ) -> Result<SkillConflict> {
        let mut store = self.store.lock().await;
        let conflict = store
            .conflicts
            .get(&id)
            .cloned()
            .context("conflict not found")?;
        let now = Utc::now();

        match input.resolution {
            ConflictResolution::UseWeb => {
                if let Some(server_skill) =
                    skills_service
                        .get(conflict.skill_name.as_str())
                        .await
                        .with_context(|| format!("load canonical skill {}", conflict.skill_name))?
                {
                    upsert_state(
                        &mut store.skill_states,
                        conflict.node_id,
                        &conflict.skill_name,
                        server_skill.content_hash,
                        now,
                    );
                }
            }
            ConflictResolution::UseAgent => {
                let upsert = SkillUpsert {
                    name: conflict.skill_name.clone(),
                    keywords: conflict.agent_keywords.clone(),
                    description: conflict.agent_description.clone(),
                    body: conflict.agent_body.clone(),
                };
                let saved = skills_service.upsert(upsert).await.with_context(|| {
                    format!("apply agent conflict resolution {}", conflict.skill_name)
                })?;
                upsert_state(
                    &mut store.skill_states,
                    conflict.node_id,
                    &conflict.skill_name,
                    saved.content_hash,
                    now,
                );
            }
        }

        if let Some(conflict_record) = store.conflicts.get_mut(&id) {
            conflict_record.resolution = Some(input.resolution);
            conflict_record.resolved_at = Some(now);
        }
        let public = conflict.as_public();
        self.persist_locked(&store).await?;
        Ok(public)
    }

    async fn persist_locked(&self, store: &MemoryRuntimeStore) -> Result<()> {
        if let Some(pg) = &self.pg {
            pg.persist_store(store).await?;
        }
        Ok(())
    }
}

impl MemoryRuntimeStore {
    fn with_seeded_artifacts(mut self) -> Self {
        if self.artifacts.is_empty() {
            self.artifacts = default_artifacts();
        }
        self
    }
}

impl PairingRecord {
    fn as_summary(&self, base_url: &str) -> PairingSessionSummary {
        PairingSessionSummary {
            id: self.id,
            channel: self.channel,
            device_name: self.device_name.clone(),
            platform: self.platform.clone(),
            agent_version: self.agent_version.clone(),
            status: self.status,
            approve_url: format!(
                "{}/system/agent-nodes/pairings/{}",
                base_url.trim_end_matches('/'),
                self.id
            ),
            expires_at: self.expires_at,
            approved_at: self.approved_at,
            exchanged_at: self.exchanged_at,
        }
    }
}

impl NodeRecord {
    fn as_public(&self, now: DateTime<Utc>) -> AgentNode {
        let status = match self.status {
            AgentNodeStatus::Revoked => AgentNodeStatus::Revoked,
            AgentNodeStatus::Pending if self.last_seen_at.is_none() => AgentNodeStatus::Pending,
            _ => match self.last_seen_at {
                Some(last_seen) if (now - last_seen).num_seconds() <= OFFLINE_AFTER_SECS => {
                    AgentNodeStatus::Online
                }
                Some(_) => AgentNodeStatus::Offline,
                None => AgentNodeStatus::Pending,
            },
        };

        AgentNode {
            id: self.id,
            display_name: self.display_name.clone(),
            platform: self.platform.clone(),
            channel: self.channel,
            agent_version: self.agent_version.clone(),
            status,
            paired_at: self.paired_at,
            last_seen_at: self.last_seen_at,
            last_sync_at: self.last_sync_at,
            last_uploaded_count: self.last_uploaded_count,
            last_downloaded_count: self.last_downloaded_count,
            last_conflict_count: self.last_conflict_count,
        }
    }
}

impl ConflictRecord {
    fn as_public(&self) -> SkillConflict {
        SkillConflict {
            id: self.id,
            node_id: self.node_id,
            skill_name: self.skill_name.clone(),
            server_hash: self.server_hash.clone(),
            agent_hash: self.agent_hash.clone(),
            server_updated_at: self.server_updated_at,
            agent_updated_at: self.agent_updated_at,
            created_at: self.created_at,
            resolved_at: self.resolved_at,
            resolution: self.resolution,
        }
    }
}

impl PgRepo {
    async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(4)
            .acquire_timeout(Duration::from_secs(3))
            .connect(database_url)
            .await
            .context("connect agent runtime postgres")?;
        Ok(Self { pool })
    }

    async fn ensure_schema(&self) -> Result<()> {
        for statement in SCHEMA_SQL.split(';') {
            let trimmed = statement.trim();
            if trimmed.is_empty() {
                continue;
            }
            sqlx::query(trimmed)
                .execute(&self.pool)
                .await
                .with_context(|| format!("apply runtime schema statement `{trimmed}`"))?;
        }
        Ok(())
    }

    async fn load_store(&self) -> Result<MemoryRuntimeStore> {
        let artifact_rows = sqlx::query(
            r#"SELECT id, channel, title, version, platform, package_format, download_url, checksum,
                      install_command, launch_command, uninstall_command, service_name, note, active
               FROM agent_artifacts
               ORDER BY channel, title"#,
        )
        .fetch_all(&self.pool)
        .await
        .context("load agent artifacts")?;

        let pairing_rows = sqlx::query(
            r#"SELECT id, channel, device_name, platform, agent_version, status, poll_token_hash,
                      expires_at, approved_at, exchanged_at
               FROM agent_pairing_sessions"#,
        )
        .fetch_all(&self.pool)
        .await
        .context("load pairing sessions")?;

        let node_rows = sqlx::query(
            r#"SELECT id, display_name, platform, channel, agent_version, status, token_hash,
                      paired_at, last_seen_at, last_sync_at, last_uploaded_count,
                      last_downloaded_count, last_conflict_count
               FROM agent_nodes"#,
        )
        .fetch_all(&self.pool)
        .await
        .context("load agent nodes")?;

        let conflict_rows = sqlx::query(
            r#"SELECT id, node_id, skill_name, server_hash, agent_hash, server_updated_at,
                      agent_updated_at, created_at, resolved_at, resolution, agent_keywords,
                      agent_description, agent_body
               FROM agent_skill_conflicts"#,
        )
        .fetch_all(&self.pool)
        .await
        .context("load skill conflicts")?;

        let state_rows = sqlx::query(
            r#"SELECT node_id, skill_name, last_synced_hash, last_synced_at
               FROM agent_skill_states"#,
        )
        .fetch_all(&self.pool)
        .await
        .context("load skill states")?;

        let artifacts = artifact_rows
            .into_iter()
            .map(|row| AgentArtifact {
                id: row.try_get("id").unwrap_or_else(|_| Uuid::new_v4()),
                channel: parse_channel(&row.try_get::<String, _>("channel").unwrap_or_default())
                    .unwrap_or(AgentArtifactChannel::MacosBinary),
                title: row.try_get("title").unwrap_or_default(),
                version: row.try_get("version").unwrap_or_default(),
                platform: row.try_get("platform").unwrap_or_default(),
                package_format: row.try_get("package_format").unwrap_or_default(),
                download_url: row.try_get("download_url").unwrap_or_default(),
                checksum: row.try_get("checksum").unwrap_or_default(),
                install_command: row.try_get("install_command").unwrap_or_default(),
                launch_command: row.try_get("launch_command").unwrap_or_default(),
                uninstall_command: row.try_get("uninstall_command").unwrap_or_default(),
                service_name: row.try_get("service_name").unwrap_or_default(),
                note: row.try_get("note").unwrap_or_default(),
                active: row.try_get("active").unwrap_or(true),
            })
            .collect::<Vec<_>>();

        let pairings = pairing_rows
            .into_iter()
            .map(|row| {
                let record = PairingRecord {
                    id: row.try_get("id").unwrap_or_else(|_| Uuid::new_v4()),
                    channel: parse_channel(
                        &row.try_get::<String, _>("channel").unwrap_or_default(),
                    )
                    .unwrap_or(AgentArtifactChannel::MacosBinary),
                    device_name: row.try_get("device_name").unwrap_or_default(),
                    platform: row.try_get("platform").unwrap_or_default(),
                    agent_version: row.try_get("agent_version").unwrap_or_default(),
                    status: parse_pairing_status(
                        &row.try_get::<String, _>("status").unwrap_or_default(),
                    )
                    .unwrap_or(PairingStatus::Pending),
                    poll_token_hash: row.try_get("poll_token_hash").unwrap_or_default(),
                    expires_at: row.try_get("expires_at").unwrap_or_else(|_| Utc::now()),
                    approved_at: row.try_get("approved_at").ok(),
                    exchanged_at: row.try_get("exchanged_at").ok(),
                };
                (record.id, record)
            })
            .collect::<BTreeMap<_, _>>();

        let nodes = node_rows
            .into_iter()
            .map(|row| {
                let record = NodeRecord {
                    id: row.try_get("id").unwrap_or_else(|_| Uuid::new_v4()),
                    display_name: row.try_get("display_name").unwrap_or_default(),
                    platform: row.try_get("platform").unwrap_or_default(),
                    channel: parse_channel(
                        &row.try_get::<String, _>("channel").unwrap_or_default(),
                    )
                    .unwrap_or(AgentArtifactChannel::MacosBinary),
                    agent_version: row.try_get("agent_version").unwrap_or_default(),
                    status: parse_node_status(
                        &row.try_get::<String, _>("status").unwrap_or_default(),
                    )
                    .unwrap_or(AgentNodeStatus::Pending),
                    token_hash: row.try_get("token_hash").unwrap_or_default(),
                    paired_at: row.try_get("paired_at").unwrap_or_else(|_| Utc::now()),
                    last_seen_at: row.try_get("last_seen_at").ok(),
                    last_sync_at: row.try_get("last_sync_at").ok(),
                    last_uploaded_count: usize::try_from(
                        row.try_get::<i64, _>("last_uploaded_count")
                            .unwrap_or_default(),
                    )
                    .unwrap_or_default(),
                    last_downloaded_count: usize::try_from(
                        row.try_get::<i64, _>("last_downloaded_count")
                            .unwrap_or_default(),
                    )
                    .unwrap_or_default(),
                    last_conflict_count: usize::try_from(
                        row.try_get::<i64, _>("last_conflict_count")
                            .unwrap_or_default(),
                    )
                    .unwrap_or_default(),
                };
                (record.id, record)
            })
            .collect::<BTreeMap<_, _>>();

        let conflicts = conflict_rows
            .into_iter()
            .map(|row| {
                let record = ConflictRecord {
                    id: row.try_get("id").unwrap_or_else(|_| Uuid::new_v4()),
                    node_id: row.try_get("node_id").unwrap_or_else(|_| Uuid::new_v4()),
                    skill_name: row.try_get("skill_name").unwrap_or_default(),
                    server_hash: row.try_get("server_hash").unwrap_or_default(),
                    agent_hash: row.try_get("agent_hash").unwrap_or_default(),
                    server_updated_at: row.try_get("server_updated_at").ok(),
                    agent_updated_at: row.try_get("agent_updated_at").ok(),
                    created_at: row.try_get("created_at").unwrap_or_else(|_| Utc::now()),
                    resolved_at: row.try_get("resolved_at").ok(),
                    resolution: row
                        .try_get::<Option<String>, _>("resolution")
                        .ok()
                        .flatten()
                        .as_deref()
                        .and_then(parse_resolution),
                    agent_keywords: row.try_get("agent_keywords").unwrap_or_default(),
                    agent_description: row.try_get("agent_description").unwrap_or_default(),
                    agent_body: row.try_get("agent_body").unwrap_or_default(),
                };
                (record.id, record)
            })
            .collect::<BTreeMap<_, _>>();

        let skill_states = state_rows
            .into_iter()
            .map(|row| {
                let node_id = row.try_get("node_id").unwrap_or_else(|_| Uuid::new_v4());
                let skill_name = row.try_get::<String, _>("skill_name").unwrap_or_default();
                let state = SkillStateRecord {
                    node_id,
                    skill_name: skill_name.clone(),
                    last_synced_hash: row.try_get("last_synced_hash").unwrap_or_default(),
                    last_synced_at: row.try_get("last_synced_at").unwrap_or_else(|_| Utc::now()),
                };
                ((node_id, skill_name), state)
            })
            .collect::<BTreeMap<_, _>>();

        Ok(MemoryRuntimeStore {
            artifacts,
            pairings,
            nodes,
            conflicts,
            skill_states,
        })
    }

    async fn persist_store(&self, store: &MemoryRuntimeStore) -> Result<()> {
        let mut tx = self.pool.begin().await.context("begin runtime tx")?;
        sqlx::query("DELETE FROM agent_skill_conflicts")
            .execute(&mut *tx)
            .await
            .context("clear conflicts")?;
        sqlx::query("DELETE FROM agent_skill_states")
            .execute(&mut *tx)
            .await
            .context("clear skill states")?;
        sqlx::query("DELETE FROM agent_pairing_sessions")
            .execute(&mut *tx)
            .await
            .context("clear pairings")?;
        sqlx::query("DELETE FROM agent_nodes")
            .execute(&mut *tx)
            .await
            .context("clear nodes")?;
        sqlx::query("DELETE FROM agent_artifacts")
            .execute(&mut *tx)
            .await
            .context("clear artifacts")?;

        for artifact in &store.artifacts {
            sqlx::query(
                r#"INSERT INTO agent_artifacts (
                    id, channel, title, version, platform, package_format, download_url,
                    checksum, install_command, launch_command, uninstall_command,
                    service_name, note, active
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7,
                    $8, $9, $10, $11,
                    $12, $13, $14
                )"#,
            )
            .bind(artifact.id)
            .bind(artifact.channel.as_str())
            .bind(&artifact.title)
            .bind(&artifact.version)
            .bind(&artifact.platform)
            .bind(&artifact.package_format)
            .bind(&artifact.download_url)
            .bind(&artifact.checksum)
            .bind(&artifact.install_command)
            .bind(&artifact.launch_command)
            .bind(&artifact.uninstall_command)
            .bind(&artifact.service_name)
            .bind(&artifact.note)
            .bind(artifact.active)
            .execute(&mut *tx)
            .await
            .context("insert artifact")?;
        }

        for pairing in store.pairings.values() {
            sqlx::query(
                r#"INSERT INTO agent_pairing_sessions (
                    id, channel, device_name, platform, agent_version, status,
                    poll_token_hash, expires_at, approved_at, exchanged_at
                ) VALUES (
                    $1, $2, $3, $4, $5, $6,
                    $7, $8, $9, $10
                )"#,
            )
            .bind(pairing.id)
            .bind(pairing.channel.as_str())
            .bind(&pairing.device_name)
            .bind(&pairing.platform)
            .bind(&pairing.agent_version)
            .bind(pairing_status_str(pairing.status))
            .bind(&pairing.poll_token_hash)
            .bind(pairing.expires_at)
            .bind(pairing.approved_at)
            .bind(pairing.exchanged_at)
            .execute(&mut *tx)
            .await
            .context("insert pairing")?;
        }

        for node in store.nodes.values() {
            sqlx::query(
                r#"INSERT INTO agent_nodes (
                    id, display_name, platform, channel, agent_version, status, token_hash,
                    paired_at, last_seen_at, last_sync_at, last_uploaded_count,
                    last_downloaded_count, last_conflict_count
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7,
                    $8, $9, $10, $11,
                    $12, $13
                )"#,
            )
            .bind(node.id)
            .bind(&node.display_name)
            .bind(&node.platform)
            .bind(node.channel.as_str())
            .bind(&node.agent_version)
            .bind(node_status_str(node.status))
            .bind(&node.token_hash)
            .bind(node.paired_at)
            .bind(node.last_seen_at)
            .bind(node.last_sync_at)
            .bind(i64::try_from(node.last_uploaded_count).unwrap_or_default())
            .bind(i64::try_from(node.last_downloaded_count).unwrap_or_default())
            .bind(i64::try_from(node.last_conflict_count).unwrap_or_default())
            .execute(&mut *tx)
            .await
            .context("insert node")?;
        }

        for state in store.skill_states.values() {
            sqlx::query(
                r#"INSERT INTO agent_skill_states (
                    node_id, skill_name, last_synced_hash, last_synced_at
                ) VALUES ($1, $2, $3, $4)"#,
            )
            .bind(state.node_id)
            .bind(&state.skill_name)
            .bind(&state.last_synced_hash)
            .bind(state.last_synced_at)
            .execute(&mut *tx)
            .await
            .context("insert skill state")?;
        }

        for conflict in store.conflicts.values() {
            sqlx::query(
                r#"INSERT INTO agent_skill_conflicts (
                    id, node_id, skill_name, server_hash, agent_hash, server_updated_at,
                    agent_updated_at, created_at, resolved_at, resolution, agent_keywords,
                    agent_description, agent_body
                ) VALUES (
                    $1, $2, $3, $4, $5, $6,
                    $7, $8, $9, $10, $11,
                    $12, $13
                )"#,
            )
            .bind(conflict.id)
            .bind(conflict.node_id)
            .bind(&conflict.skill_name)
            .bind(&conflict.server_hash)
            .bind(&conflict.agent_hash)
            .bind(conflict.server_updated_at)
            .bind(conflict.agent_updated_at)
            .bind(conflict.created_at)
            .bind(conflict.resolved_at)
            .bind(conflict.resolution.map(resolution_str))
            .bind(&conflict.agent_keywords)
            .bind(&conflict.agent_description)
            .bind(&conflict.agent_body)
            .execute(&mut *tx)
            .await
            .context("insert conflict")?;
        }

        tx.commit().await.context("commit runtime tx")?;
        Ok(())
    }
}

fn default_artifacts() -> Vec<AgentArtifact> {
    vec![
        AgentArtifact {
            id: Uuid::new_v4(),
            channel: AgentArtifactChannel::MacosBinary,
            title: "Agent Runtime for macOS".into(),
            version: std::env::var("ADDZERO_AGENT_MACOS_VERSION")
                .unwrap_or_else(|_| "0.1.0".into()),
            platform: "macOS / arm64".into(),
            package_format: "binary + launchd".into(),
            download_url: std::env::var("ADDZERO_AGENT_MACOS_DOWNLOAD_URL")
                .unwrap_or_else(|_| "https://example.invalid/agent-runtime-macos-arm64.zip".into()),
            checksum: std::env::var("ADDZERO_AGENT_MACOS_SHA256")
                .unwrap_or_else(|_| "pending".into()),
            install_command: "curl -L \"$URL\" -o agent-runtime.zip && unzip agent-runtime.zip && ./install-agent-runtime.sh".into(),
            launch_command: "launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/site.addzero.agent-runtime.plist".into(),
            uninstall_command: "launchctl bootout gui/$(id -u) ~/Library/LaunchAgents/site.addzero.agent-runtime.plist && rm -f ~/Library/LaunchAgents/site.addzero.agent-runtime.plist".into(),
            service_name: "site.addzero.agent-runtime".into(),
            note: "推荐形态：后台静默运行，可重装、可卸载。".into(),
            active: true,
        },
        AgentArtifact {
            id: Uuid::new_v4(),
            channel: AgentArtifactChannel::DockerCompose,
            title: "Agent Runtime via docker compose".into(),
            version: std::env::var("ADDZERO_AGENT_COMPOSE_VERSION")
                .unwrap_or_else(|_| "0.1.0".into()),
            platform: "Docker / cross-platform".into(),
            package_format: "compose bundle".into(),
            download_url: std::env::var("ADDZERO_AGENT_COMPOSE_DOWNLOAD_URL")
                .unwrap_or_else(|_| "https://example.invalid/agent-runtime-compose.tgz".into()),
            checksum: std::env::var("ADDZERO_AGENT_COMPOSE_SHA256")
                .unwrap_or_else(|_| "pending".into()),
            install_command: "curl -L \"$URL\" -o agent-runtime-compose.tgz && tar -xzf agent-runtime-compose.tgz".into(),
            launch_command: "docker compose up -d".into(),
            uninstall_command: "docker compose down -v".into(),
            service_name: "agent-runtime".into(),
            note: "兼容交付面：适合 NAS、Linux 主机或容器环境。".into(),
            active: true,
        },
    ]
}

fn find_node_mut<'a>(
    nodes: &'a mut BTreeMap<Uuid, NodeRecord>,
    node_token: &str,
) -> Result<&'a mut NodeRecord> {
    let token_hash = hash_secret(node_token);
    nodes
        .values_mut()
        .find(|node| node.token_hash == token_hash && node.status != AgentNodeStatus::Revoked)
        .ok_or_else(|| anyhow!("agent node token is invalid"))
}

fn upsert_state(
    states: &mut BTreeMap<(Uuid, String), SkillStateRecord>,
    node_id: Uuid,
    skill_name: &str,
    hash: String,
    synced_at: DateTime<Utc>,
) {
    states.insert(
        (node_id, skill_name.to_string()),
        SkillStateRecord {
            node_id,
            skill_name: skill_name.to_string(),
            last_synced_hash: hash,
            last_synced_at: synced_at,
        },
    );
}

fn upsert_conflict(
    conflicts: &mut BTreeMap<Uuid, ConflictRecord>,
    node_id: Uuid,
    server_skill: &Skill,
    local_skill: &SkillSnapshot,
) -> SkillConflict {
    if let Some(conflict) = conflicts.values_mut().find(|conflict| {
        conflict.node_id == node_id
            && conflict.skill_name == server_skill.name
            && conflict.resolved_at.is_none()
    }) {
        conflict.server_hash = server_skill.content_hash.clone();
        conflict.agent_hash = local_skill.content_hash.clone();
        conflict.server_updated_at = Some(server_skill.updated_at);
        conflict.agent_updated_at = local_skill.updated_at;
        conflict.agent_keywords = local_skill.keywords.clone();
        conflict.agent_description = local_skill.description.clone();
        conflict.agent_body = local_skill.body.clone();
        return conflict.as_public();
    }

    let conflict = ConflictRecord {
        id: Uuid::new_v4(),
        node_id,
        skill_name: server_skill.name.clone(),
        server_hash: server_skill.content_hash.clone(),
        agent_hash: local_skill.content_hash.clone(),
        server_updated_at: Some(server_skill.updated_at),
        agent_updated_at: local_skill.updated_at,
        created_at: Utc::now(),
        resolved_at: None,
        resolution: None,
        agent_keywords: local_skill.keywords.clone(),
        agent_description: local_skill.description.clone(),
        agent_body: local_skill.body.clone(),
    };
    let public = conflict.as_public();
    conflicts.insert(conflict.id, conflict);
    public
}

fn snapshot_to_upsert(snapshot: &SkillSnapshot) -> SkillUpsert {
    SkillUpsert {
        name: snapshot.name.clone(),
        keywords: snapshot.keywords.clone(),
        description: snapshot.description.clone(),
        body: snapshot.body.clone(),
    }
}

fn skill_to_snapshot(skill: &Skill) -> SkillSnapshot {
    SkillSnapshot {
        name: skill.name.clone(),
        keywords: skill.keywords.clone(),
        description: skill.description.clone(),
        body: skill.body.clone(),
        content_hash: skill.content_hash.clone(),
        updated_at: Some(skill.updated_at),
    }
}

fn normalize_pairings(pairings: &mut BTreeMap<Uuid, PairingRecord>) {
    let now = Utc::now();
    for pairing in pairings.values_mut() {
        if pairing.status == PairingStatus::Pending && pairing.expires_at < now {
            pairing.status = PairingStatus::Expired;
        }
    }
}

fn hash_secret(raw: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn parse_channel(raw: &str) -> Option<AgentArtifactChannel> {
    match raw {
        "macos_binary" => Some(AgentArtifactChannel::MacosBinary),
        "docker_compose" => Some(AgentArtifactChannel::DockerCompose),
        _ => None,
    }
}

fn parse_pairing_status(raw: &str) -> Option<PairingStatus> {
    match raw {
        "pending" => Some(PairingStatus::Pending),
        "approved" => Some(PairingStatus::Approved),
        "exchanged" => Some(PairingStatus::Exchanged),
        "expired" => Some(PairingStatus::Expired),
        "revoked" => Some(PairingStatus::Revoked),
        _ => None,
    }
}

fn parse_node_status(raw: &str) -> Option<AgentNodeStatus> {
    match raw {
        "pending" => Some(AgentNodeStatus::Pending),
        "online" => Some(AgentNodeStatus::Online),
        "offline" => Some(AgentNodeStatus::Offline),
        "revoked" => Some(AgentNodeStatus::Revoked),
        _ => None,
    }
}

fn parse_resolution(raw: &str) -> Option<ConflictResolution> {
    match raw {
        "use_web" => Some(ConflictResolution::UseWeb),
        "use_agent" => Some(ConflictResolution::UseAgent),
        _ => None,
    }
}

fn pairing_status_str(status: PairingStatus) -> &'static str {
    match status {
        PairingStatus::Pending => "pending",
        PairingStatus::Approved => "approved",
        PairingStatus::Exchanged => "exchanged",
        PairingStatus::Expired => "expired",
        PairingStatus::Revoked => "revoked",
    }
}

fn node_status_str(status: AgentNodeStatus) -> &'static str {
    match status {
        AgentNodeStatus::Pending => "pending",
        AgentNodeStatus::Online => "online",
        AgentNodeStatus::Offline => "offline",
        AgentNodeStatus::Revoked => "revoked",
    }
}

fn resolution_str(resolution: ConflictResolution) -> &'static str {
    match resolution {
        ConflictResolution::UseWeb => "use_web",
        ConflictResolution::UseAgent => "use_agent",
    }
}

#[cfg(test)]
mod tests {
    use super::{AgentRuntimeService, default_artifacts};
    use addzero_agent_runtime_contract::{
        AgentArtifactChannel, PairingExchangeRequest, PairingRequest,
    };

    #[tokio::test]
    async fn new_pairing_should_revoke_previous_node_on_exchange() {
        let service = AgentRuntimeService::try_attach(None, "http://localhost:8787".into()).await;
        assert_eq!(default_artifacts().len(), 2);

        let first = service
            .create_pairing(PairingRequest {
                channel: AgentArtifactChannel::MacosBinary,
                device_name: "mac-mini".into(),
                platform: "macOS".into(),
                agent_version: "0.1.0".into(),
            })
            .await
            .expect("first pairing should succeed");
        service
            .approve_pairing(first.session.id)
            .await
            .expect("approval should succeed");
        let first_node = service
            .exchange_pairing(
                first.session.id,
                PairingExchangeRequest {
                    poll_token: first.poll_token.clone(),
                },
            )
            .await
            .expect("exchange should succeed");

        let second = service
            .create_pairing(PairingRequest {
                channel: AgentArtifactChannel::DockerCompose,
                device_name: "nas".into(),
                platform: "linux".into(),
                agent_version: "0.1.1".into(),
            })
            .await
            .expect("second pairing should succeed");
        service
            .approve_pairing(second.session.id)
            .await
            .expect("second approval should succeed");
        let second_node = service
            .exchange_pairing(
                second.session.id,
                PairingExchangeRequest {
                    poll_token: second.poll_token,
                },
            )
            .await
            .expect("second exchange should succeed");

        let overview = service
            .overview("~/.agents/skills".into(), false)
            .await
            .expect("overview should load");
        assert_eq!(
            overview.active_node.map(|node| node.id),
            Some(second_node.node.id)
        );
        assert_ne!(first_node.node.id, second_node.node.id);
    }
}
