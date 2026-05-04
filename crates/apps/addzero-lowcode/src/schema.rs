use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A lowcode layout — the top-level container of component nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    pub id: Uuid,
    pub name: String,
    pub nodes: Vec<Node>,
    pub created_at: String,
    pub updated_at: String,
}

/// A single component node in the layout tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub component_type: String,
    pub props: serde_json::Value,
    pub children: Vec<Node>,
    pub grid_pos: Option<GridPos>,
}

/// Position within a CSS Grid layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridPos {
    pub col: u32,
    pub row: u32,
    pub col_span: u32,
    pub row_span: u32,
}

/// Binds a component event to an action handler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBinding {
    pub event_name: String,
    pub handler: EventHandler,
}

/// The action to perform when an event fires.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum EventHandler {
    Navigate(String),
    Script(String),
    Callback(String),
}

/// Metadata for a registered component type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDef {
    pub type_name: String,
    pub props_schema: serde_json::Value,
    pub category: String,
}

/// A reusable layout template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: Uuid,
    pub name: String,
    pub layout: Layout,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_grid_pos_roundtrip() {
        let pos = GridPos {
            col: 1,
            row: 2,
            col_span: 3,
            row_span: 4,
        };
        let json = serde_json::to_string(&pos).unwrap();
        let back: GridPos = serde_json::from_str(&json).unwrap();
        assert_eq!(back.col, 1);
        assert_eq!(back.row_span, 4);
    }
}
