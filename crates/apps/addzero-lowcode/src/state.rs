use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::events::HandlerRegistry;
use crate::registry::ComponentRegistry;
use crate::scripting::ScriptEngine;

/// Shared application state for the lowcode service.
///
/// Houses the PG pool plus in-memory registries and engines.
/// The `ComponentRegistry` is wrapped in `Arc<RwLock<_>>` so the axum
/// handlers can mutate it concurrently.
#[derive(Clone)]
pub struct LowcodeState {
    pub db: PgPool,
    pub registry: Arc<RwLock<ComponentRegistry>>,
    pub script_engine: ScriptEngine,
    pub handler_registry: HandlerRegistry,
}

impl LowcodeState {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            registry: Arc::new(RwLock::new(ComponentRegistry::with_builtins())),
            script_engine: ScriptEngine::new(),
            handler_registry: HandlerRegistry::new(),
        }
    }
}
