use std::collections::HashMap;

use crate::schema::ComponentDef;

/// In-memory component type registry (skeleton — to be fleshed out in #77).
#[derive(Clone)]
pub struct ComponentRegistry {
    components: HashMap<String, ComponentDef>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Register a new component definition.
    pub fn register(&mut self, def: ComponentDef) {
        self.components.insert(def.type_name.clone(), def);
    }

    /// Look up a component by type name.
    pub fn get(&self, type_name: &str) -> Option<&ComponentDef> {
        self.components.get(type_name)
    }

    /// List all registered component definitions.
    pub fn list(&self) -> Vec<&ComponentDef> {
        self.components.values().collect()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
