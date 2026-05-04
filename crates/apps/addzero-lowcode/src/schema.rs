//! Core Schema definitions for the lowcode platform.
//!
//! Three-layer data model: Layout, Component, EventBinding.
//! All types derive serde for JSON round-trip and PG JSONB storage.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Layout schema — the top-level design document
// ---------------------------------------------------------------------------

/// A lowcode layout: grid definition + component tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutSchema {
    pub grid: GridDefinition,
    pub children: Vec<ComponentNode>,
}

/// CSS Grid container parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridDefinition {
    pub columns: u32,
    pub rows: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_height: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breakpoints: Vec<Breakpoint>,
}

/// Responsive CSS Grid override applied below a max viewport width.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Breakpoint {
    pub name: String,
    pub max_width: String,
    pub columns: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gap: Option<String>,
}

// ---------------------------------------------------------------------------
// Component tree — recursive nodes placed on the grid
// ---------------------------------------------------------------------------

/// A single component node in the layout tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentNode {
    pub id: String,
    pub type_key: String,
    pub props: serde_json::Value,
    pub grid_area: GridArea,
    #[serde(default)]
    pub children: Vec<ComponentNode>,
}

/// Position within a CSS Grid layout (1-indexed, inclusive).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridArea {
    pub col_start: u32,
    pub col_end: u32,
    pub row_start: u32,
    pub row_end: u32,
}

// ---------------------------------------------------------------------------
// Event bindings — component event → handler coupling
// ---------------------------------------------------------------------------

/// The action to perform when a component event fires.
///
/// Uses adjacently-tagged serde encoding: `{"type":"…","config":{…}}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum HandlerType {
    RhaiScript {
        script: String,
    },
    HttpCall {
        url: String,
        method: String,
        body_template: String,
    },
    EmitEvent {
        event_name: String,
    },
    Noop,
}

/// Persisted record for an event binding row (PG `lc_event_binding`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventBindingRecord {
    pub id: Uuid,
    pub layout_id: Uuid,
    pub component_path: String,
    pub event_type: String,
    pub handler_type: HandlerType,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Component registry — type metadata
// ---------------------------------------------------------------------------

/// Persisted record for a registered component type (PG `lc_component`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentDefRecord {
    pub id: Uuid,
    pub type_key: String,
    pub props_schema: serde_json::Value,
    pub category: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Tests — serde round-trip coverage
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_area_roundtrip() {
        let ga = GridArea {
            col_start: 1,
            col_end: 3,
            row_start: 2,
            row_end: 4,
        };
        let json = serde_json::to_string(&ga).unwrap();
        let back: GridArea = serde_json::from_str(&json).unwrap();
        assert_eq!(ga, back);
    }

    #[test]
    fn component_node_roundtrip() {
        let node = ComponentNode {
            id: "btn-1".into(),
            type_key: "button".into(),
            props: serde_json::json!({ "label": "Click me" }),
            grid_area: GridArea {
                col_start: 1,
                col_end: 2,
                row_start: 1,
                row_end: 2,
            },
            children: vec![],
        };
        let json = serde_json::to_string(&node).unwrap();
        let back: ComponentNode = serde_json::from_str(&json).unwrap();
        assert_eq!(node, back);
    }

    #[test]
    fn component_node_nested_roundtrip() {
        let child = ComponentNode {
            id: "inner".into(),
            type_key: "text".into(),
            props: serde_json::json!({ "content": "hello" }),
            grid_area: GridArea {
                col_start: 1,
                col_end: 2,
                row_start: 1,
                row_end: 2,
            },
            children: vec![],
        };
        let parent = ComponentNode {
            id: "container".into(),
            type_key: "div".into(),
            props: serde_json::json!({}),
            grid_area: GridArea {
                col_start: 1,
                col_end: 4,
                row_start: 1,
                row_end: 4,
            },
            children: vec![child],
        };
        let json = serde_json::to_string(&parent).unwrap();
        let back: ComponentNode = serde_json::from_str(&json).unwrap();
        assert_eq!(parent, back);
        assert_eq!(back.children.len(), 1);
    }

    #[test]
    fn layout_schema_roundtrip() {
        let layout = LayoutSchema {
            grid: GridDefinition {
                columns: 12,
                rows: 8,
                gap: Some("10px".into()),
                row_height: None,
                breakpoints: vec![],
            },
            children: vec![ComponentNode {
                id: "root".into(),
                type_key: "container".into(),
                props: serde_json::json!({}),
                grid_area: GridArea {
                    col_start: 1,
                    col_end: 13,
                    row_start: 1,
                    row_end: 9,
                },
                children: vec![],
            }],
        };
        let json = serde_json::to_string(&layout).unwrap();
        let back: LayoutSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(layout, back);
    }

    #[test]
    fn handler_type_rhai_roundtrip() {
        let h = HandlerType::RhaiScript {
            script: "print(42)".into(),
        };
        let json = serde_json::to_string(&h).unwrap();
        assert!(json.contains(r#""type":"RhaiScript""#));
        let back: HandlerType = serde_json::from_str(&json).unwrap();
        assert_eq!(h, back);
    }

    #[test]
    fn handler_type_http_roundtrip() {
        let h = HandlerType::HttpCall {
            url: "https://api.example.com".into(),
            method: "POST".into(),
            body_template: r#"{"key":"{{value}}"}"#.into(),
        };
        let json = serde_json::to_string(&h).unwrap();
        assert!(json.contains(r#""type":"HttpCall""#));
        let back: HandlerType = serde_json::from_str(&json).unwrap();
        assert_eq!(h, back);
    }

    #[test]
    fn handler_type_emit_roundtrip() {
        let h = HandlerType::EmitEvent {
            event_name: "onSubmit".into(),
        };
        let json = serde_json::to_string(&h).unwrap();
        assert!(json.contains(r#""type":"EmitEvent""#));
        let back: HandlerType = serde_json::from_str(&json).unwrap();
        assert_eq!(h, back);
    }

    #[test]
    fn handler_type_noop_roundtrip() {
        let h = HandlerType::Noop;
        let json = serde_json::to_string(&h).unwrap();
        assert!(json.contains(r#""type":"Noop""#));
        let back: HandlerType = serde_json::from_str(&json).unwrap();
        assert_eq!(h, back);
    }

    #[test]
    fn event_binding_record_roundtrip() {
        let rec = EventBindingRecord {
            id: Uuid::nil(),
            layout_id: Uuid::nil(),
            component_path: "root/0/2".into(),
            event_type: "onClick".into(),
            handler_type: HandlerType::Noop,
            created_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&rec).unwrap();
        let back: EventBindingRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(rec, back);
    }

    #[test]
    fn component_def_record_roundtrip() {
        let rec = ComponentDefRecord {
            id: Uuid::nil(),
            type_key: "button".into(),
            props_schema: serde_json::json!({ "label": "string" }),
            category: "basic".into(),
            icon: Some("click".into()),
            created_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&rec).unwrap();
        let back: ComponentDefRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(rec, back);
    }

    #[test]
    fn component_def_record_no_icon_roundtrip() {
        let rec = ComponentDefRecord {
            id: Uuid::nil(),
            type_key: "div".into(),
            props_schema: serde_json::json!({}),
            category: "layout".into(),
            icon: None,
            created_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&rec).unwrap();
        assert!(!json.contains("icon"));
        let back: ComponentDefRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(rec, back);
    }
}
