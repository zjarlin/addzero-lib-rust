use std::rc::Rc;

use addzero_agent_runtime_contract::{
    AgentRuntimeOverview, PairingSessionSummary, ResolveConflictRequest, SkillConflict,
};
use thiserror::Error;
use uuid::Uuid;

use super::skills::LocalBoxFuture;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum AgentRuntimeError {
    #[error("{0}")]
    Message(String),
}

impl AgentRuntimeError {
    fn new(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

pub type AgentRuntimeResult<T> = Result<T, AgentRuntimeError>;

pub trait AgentRuntimeApi: 'static {
    fn overview(&self) -> LocalBoxFuture<'_, AgentRuntimeResult<AgentRuntimeOverview>>;

    fn get_pairing(
        &self,
        id: Uuid,
    ) -> LocalBoxFuture<'_, AgentRuntimeResult<PairingSessionSummary>>;

    fn approve_pairing(
        &self,
        id: Uuid,
    ) -> LocalBoxFuture<'_, AgentRuntimeResult<PairingSessionSummary>>;

    fn resolve_conflict(
        &self,
        id: Uuid,
        input: ResolveConflictRequest,
    ) -> LocalBoxFuture<'_, AgentRuntimeResult<SkillConflict>>;
}

pub type SharedAgentRuntimeApi = Rc<dyn AgentRuntimeApi>;

pub fn default_agent_runtime_api() -> SharedAgentRuntimeApi {
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(BrowserAgentRuntimeApi)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(EmbeddedAgentRuntimeApi)
    }
}

#[cfg(target_arch = "wasm32")]
struct BrowserAgentRuntimeApi;

#[cfg(target_arch = "wasm32")]
impl AgentRuntimeApi for BrowserAgentRuntimeApi {
    fn overview(&self) -> LocalBoxFuture<'_, AgentRuntimeResult<AgentRuntimeOverview>> {
        Box::pin(async move {
            super::browser_http::get_json("/api/runtime/overview")
                .await
                .map_err(AgentRuntimeError::new)
        })
    }

    fn get_pairing(
        &self,
        id: Uuid,
    ) -> LocalBoxFuture<'_, AgentRuntimeResult<PairingSessionSummary>> {
        Box::pin(async move {
            super::browser_http::get_json(&format!("/api/runtime/pairings/{id}"))
                .await
                .map_err(AgentRuntimeError::new)
        })
    }

    fn approve_pairing(
        &self,
        id: Uuid,
    ) -> LocalBoxFuture<'_, AgentRuntimeResult<PairingSessionSummary>> {
        Box::pin(async move {
            let payload = serde_json::json!({});
            super::browser_http::post_json(&format!("/api/runtime/pairings/{id}/approve"), &payload)
                .await
                .map_err(AgentRuntimeError::new)
        })
    }

    fn resolve_conflict(
        &self,
        id: Uuid,
        input: ResolveConflictRequest,
    ) -> LocalBoxFuture<'_, AgentRuntimeResult<SkillConflict>> {
        Box::pin(async move {
            super::browser_http::post_json(&format!("/api/runtime/conflicts/{id}/resolve"), &input)
                .await
                .map_err(AgentRuntimeError::new)
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct EmbeddedAgentRuntimeApi;

#[cfg(not(target_arch = "wasm32"))]
impl AgentRuntimeApi for EmbeddedAgentRuntimeApi {
    fn overview(&self) -> LocalBoxFuture<'_, AgentRuntimeResult<AgentRuntimeOverview>> {
        Box::pin(async move {
            let backend = crate::server::services().await;
            backend
                .runtime
                .overview(
                    backend.skills.fs_root_display(),
                    backend.skills.is_pg_online(),
                )
                .await
                .map_err(|err| AgentRuntimeError::new(err.to_string()))
        })
    }

    fn get_pairing(
        &self,
        id: Uuid,
    ) -> LocalBoxFuture<'_, AgentRuntimeResult<PairingSessionSummary>> {
        Box::pin(async move {
            let backend = crate::server::services().await;
            backend
                .runtime
                .get_pairing(id, None)
                .await
                .map_err(|err| AgentRuntimeError::new(err.to_string()))
        })
    }

    fn approve_pairing(
        &self,
        id: Uuid,
    ) -> LocalBoxFuture<'_, AgentRuntimeResult<PairingSessionSummary>> {
        Box::pin(async move {
            let backend = crate::server::services().await;
            backend
                .runtime
                .approve_pairing(id)
                .await
                .map_err(|err| AgentRuntimeError::new(err.to_string()))
        })
    }

    fn resolve_conflict(
        &self,
        id: Uuid,
        input: ResolveConflictRequest,
    ) -> LocalBoxFuture<'_, AgentRuntimeResult<SkillConflict>> {
        Box::pin(async move {
            let backend = crate::server::services().await;
            backend
                .runtime
                .resolve_conflict(id, input, &backend.skills)
                .await
                .map_err(|err| AgentRuntimeError::new(err.to_string()))
        })
    }
}
