//! Shared application state for the lowcode service.
//!
//! Houses the PG pool, in-memory layout store, registries, and engines.

use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::events::HandlerRegistry;
use crate::registry::ComponentRegistry;
use crate::schema::LayoutSchema;
use crate::scripting::ScriptEngine;

/// Shared application state for the lowcode service.
///
/// Houses the PG pool plus in-memory registries and engines.
/// The `ComponentRegistry` is wrapped in `Arc<RwLock<_>>` so the axum
/// handlers can mutate it concurrently.
///
/// `layouts` is a temporary in-memory store that will be replaced by PG-backed
/// `LayoutRepository` once layout CRUD is fully implemented.
#[derive(Clone)]
pub struct LowcodeState {
    pub db: PgPool,
    pub registry: Arc<RwLock<ComponentRegistry>>,
    pub script_engine: ScriptEngine,
    pub handler_registry: HandlerRegistry,
    /// Temporary in-memory layout store (will be backed by PG repository).
    pub layouts: Arc<RwLock<HashMap<Uuid, LayoutSchema>>>,
}

impl LowcodeState {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            registry: Arc::new(RwLock::new(ComponentRegistry::with_builtins())),
            script_engine: ScriptEngine::new(),
            handler_registry: HandlerRegistry::new(),
            layouts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
