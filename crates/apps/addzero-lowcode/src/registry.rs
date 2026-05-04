use std::collections::HashMap;

use crate::schema::ComponentDefRecord;

/// In-memory component type registry (skeleton — to be fleshed out in #77).
#[derive(Clone)]
pub struct ComponentRegistry {
    components: HashMap<String, ComponentDefRecord>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Register a new component definition.
    pub fn register(&mut self, def: ComponentDefRecord) {
        self.components.insert(def.type_key.clone(), def);
    }

    /// Look up a component by type key.
    pub fn get(&self, type_key: &str) -> Option<&ComponentDefRecord> {
        self.components.get(type_key)
    }

    /// List all registered component definitions.
    pub fn list(&self) -> Vec<&ComponentDefRecord> {
        self.components.values().collect()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
