use sqlx::PgPool;

use crate::events::HandlerRegistry;
use crate::registry::ComponentRegistry;
use crate::scripting::ScriptEngine;

/// Shared application state for the lowcode service.
///
/// Houses the PG pool plus in-memory registries and engines.
#[derive(Clone)]
pub struct LowcodeState {
    pub db: PgPool,
    pub registry: ComponentRegistry,
    pub script_engine: ScriptEngine,
    pub handler_registry: HandlerRegistry,
}

impl LowcodeState {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            registry: ComponentRegistry::new(),
            script_engine: ScriptEngine::new(),
            handler_registry: HandlerRegistry::new(),
        }
    }
}
