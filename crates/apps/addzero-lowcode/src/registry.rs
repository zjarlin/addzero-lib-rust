//! Component registry — runtime type registration, props validation, and rendering.
//!
//! Manages `ComponentEntry` objects that pair a JSON Schema definition with a
//! renderer closure. Ships with 8 built-in component types (button, input, text,
//! container, table, form, image, divider).

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::schema::{ComponentDefRecord, ComponentNode};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Closure that renders a `ComponentNode` to an HTML string.
pub type ComponentRenderer = Box<dyn Fn(&ComponentNode) -> String + Send + Sync>;

/// Runtime entry for a registered component type.
///
/// Combines metadata (type key, category) with a JSON Schema describing
/// accepted props and a renderer closure that produces HTML output.
pub struct ComponentEntry {
    pub type_key: String,
    pub category: String,
    pub props_schema: serde_json::Value,
    pub renderer: ComponentRenderer,
}

/// Errors returned by the component registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryError(pub Vec<String>);

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "registry errors: {}", self.0.join("; "))
    }
}

impl std::error::Error for RegistryError {}

/// Lightweight JSON view returned by the list API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub type_key: String,
    pub category: String,
    pub props_schema: serde_json::Value,
}

// ---------------------------------------------------------------------------
// ComponentRegistry
// ---------------------------------------------------------------------------

/// In-memory component type registry.
///
/// Stores `ComponentEntry` instances keyed by `type_key`. Supports CRUD
/// operations, JSON Schema prop validation, and node rendering.
pub struct ComponentRegistry {
    entries: HashMap<String, ComponentEntry>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Register a runtime component entry.
    pub fn register(&mut self, entry: ComponentEntry) {
        self.entries.insert(entry.type_key.clone(), entry);
    }

    /// Create a `ComponentEntry` from a persisted DB record and register it.
    pub fn register_from_record(
        &mut self,
        record: ComponentDefRecord,
        renderer: ComponentRenderer,
    ) {
        let entry = ComponentEntry {
            type_key: record.type_key,
            category: record.category,
            props_schema: record.props_schema,
            renderer,
        };
        self.entries.insert(entry.type_key.clone(), entry);
    }

    /// Remove a component type. Returns `true` if it existed.
    pub fn unregister(&mut self, type_key: &str) -> bool {
        self.entries.remove(type_key).is_some()
    }

    /// Look up a component entry by type key.
    pub fn get_entry(&self, type_key: &str) -> Option<&ComponentEntry> {
        self.entries.get(type_key)
    }

    /// Alias for `get_entry` (backward compatibility).
    pub fn get(&self, type_key: &str) -> Option<&ComponentEntry> {
        self.get_entry(type_key)
    }

    /// List all registered entries.
    pub fn list(&self) -> Vec<&ComponentEntry> {
        self.entries.values().collect()
    }

    /// List entries filtered by category.
    pub fn list_by_category(&self, category: &str) -> Vec<&ComponentEntry> {
        self.entries
            .values()
            .filter(|e| e.category == category)
            .collect()
    }

    /// Collect distinct categories (unsorted).
    pub fn categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self
            .entries
            .values()
            .map(|e| e.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        cats.sort();
        cats
    }

    /// Return lightweight info for every registered entry (used by the API).
    pub fn list_info(&self) -> Vec<ComponentInfo> {
        self.entries
            .values()
            .map(|e| ComponentInfo {
                type_key: e.type_key.clone(),
                category: e.category.clone(),
                props_schema: e.props_schema.clone(),
            })
            .collect()
    }

    /// Validate `props` against the JSON Schema stored for `type_key`.
    ///
    /// Returns `Ok(())` on success, or `Err(Vec<String>)` with one message per
    /// validation failure. Supported checks:
    /// - **required** fields are present
    /// - basic **type** matching (string / number / boolean / array / object)
    pub fn validate_props(
        &self,
        type_key: &str,
        props: &serde_json::Value,
    ) -> Result<(), Vec<String>> {
        let entry = self
            .entries
            .get(type_key)
            .ok_or_else(|| vec![format!("unknown component type: {type_key}")])?;

        let schema = &entry.props_schema;
        let mut errors = Vec::new();

        // Collect required fields
        if let Some(required) = schema.get("required").and_then(|v| v.as_array()) {
            for field in required {
                if let Some(name) = field.as_str() {
                    if props.get(name).is_none() || props.get(name).unwrap().is_null() {
                        errors.push(format!("missing required field: {name}"));
                    }
                }
            }
        }

        // Type-check each declared property
        if let Some(properties) = schema.get("properties").and_then(|v| v.as_object()) {
            for (field_name, field_schema) in properties {
                if let Some(value) = props.get(field_name) {
                    if value.is_null() {
                        continue; // null is allowed for optional fields
                    }
                    if let Some(expected_type) = field_schema.get("type").and_then(|v| v.as_str()) {
                        let ok = match expected_type {
                            "string" => value.is_string(),
                            "number" => value.is_number(),
                            "integer" => value.is_i64() || value.is_u64(),
                            "boolean" => value.is_boolean(),
                            "array" => value.is_array(),
                            "object" => value.is_object(),
                            _ => true, // unknown types pass
                        };
                        if !ok {
                            errors.push(format!(
                                "field '{field_name}': expected {expected_type}, got {}",
                                json_type_name(value),
                            ));
                        }
                    }

                    // Enum validation
                    if let Some(enum_vals) = field_schema.get("enum").and_then(|v| v.as_array()) {
                        if !enum_vals.contains(value) {
                            errors.push(format!(
                                "field '{field_name}': value {value} not in enum {:?}",
                                enum_vals,
                            ));
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Render a component node to HTML by looking up its type and invoking the
    /// registered renderer.
    pub fn render(&self, node: &ComponentNode) -> Result<String, String> {
        let entry = self
            .entries
            .get(&node.type_key)
            .ok_or_else(|| format!("unknown component type: {}", node.type_key))?;
        Ok((entry.renderer)(node))
    }

    /// Create a registry pre-loaded with the 8 built-in component types.
    pub fn with_builtins() -> Self {
        let mut reg = Self::new();
        register_builtins(&mut reg);
        reg
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn json_type_name(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

/// Extract a string prop, falling back to `default`.
fn str_prop(props: &serde_json::Value, key: &str, default: &str) -> String {
    props
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}

/// Extract a boolean prop, falling back to `default`.
fn bool_prop(props: &serde_json::Value, key: &str, default: bool) -> bool {
    props.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}

/// Register the 8 built-in component types.
fn register_builtins(reg: &mut ComponentRegistry) {
    // ---- button ----
    reg.register(ComponentEntry {
        type_key: "button".into(),
        category: "basic".into(),
        props_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "label":    { "type": "string" },
                "variant":  { "type": "string", "enum": ["primary", "secondary", "danger"], "default": "primary" },
                "disabled": { "type": "boolean", "default": false }
            },
            "required": ["label"]
        }),
        renderer: Box::new(|node| {
            let p = &node.props;
            let label = str_prop(p, "label", "");
            let variant = str_prop(p, "variant", "primary");
            let disabled = bool_prop(p, "disabled", false);
            let dis_attr = if disabled { " disabled" } else { "" };
            format!(
                r#"<button class="lc-button lc-button--{variant}"{dis_attr}>{label}</button>"#
            )
        }),
    });

    // ---- input ----
    reg.register(ComponentEntry {
        type_key: "input".into(),
        category: "basic".into(),
        props_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "placeholder": { "type": "string" },
                "input_type":  { "type": "string", "enum": ["text", "email", "password", "number"], "default": "text" },
                "required":    { "type": "boolean" }
            },
            "required": []
        }),
        renderer: Box::new(|node| {
            let p = &node.props;
            let placeholder = str_prop(p, "placeholder", "");
            let input_type = str_prop(p, "input_type", "text");
            let required = bool_prop(p, "required", false);
            let req_attr = if required { " required" } else { "" };
            format!(
                r#"<input type="{input_type}" placeholder="{placeholder}"{req_attr} />"#
            )
        }),
    });

    // ---- text ----
    reg.register(ComponentEntry {
        type_key: "text".into(),
        category: "basic".into(),
        props_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "content": { "type": "string" },
                "tag":     { "type": "string", "enum": ["p", "h1", "h2", "h3", "h4", "span"], "default": "p" },
                "align":   { "type": "string", "enum": ["left", "center", "right"], "default": "left" }
            },
            "required": ["content"]
        }),
        renderer: Box::new(|node| {
            let p = &node.props;
            let content = str_prop(p, "content", "");
            let tag = str_prop(p, "tag", "p");
            let align = str_prop(p, "align", "left");
            format!(
                r#"<{tag} style="text-align:{align}">{content}</{tag}>"#
            )
        }),
    });

    // ---- container ----
    reg.register(ComponentEntry {
        type_key: "container".into(),
        category: "layout".into(),
        props_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "direction": { "type": "string", "enum": ["row", "column"], "default": "column" },
                "padding":   { "type": "string" }
            },
            "required": []
        }),
        renderer: Box::new(|node| {
            let p = &node.props;
            let direction = str_prop(p, "direction", "column");
            let padding = str_prop(p, "padding", "0");
            let children_html = node
                .children
                .iter()
                .map(|child| {
                    // Inline mini-render: produce a placeholder for children
                    // that aren't rendered through the full registry path.
                    format!(
                        r#"<div class="lc-child" data-id="{}">{}</div>"#,
                        child.id, child.props
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            format!(
                r#"<div class="lc-container" style="display:flex;flex-direction:{direction};padding:{padding}">{children_html}</div>"#
            )
        }),
    });

    // ---- table ----
    reg.register(ComponentEntry {
        type_key: "table".into(),
        category: "data".into(),
        props_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "columns":     { "type": "array" },
                "data_source": { "type": "string" },
                "pagination":  { "type": "boolean" }
            },
            "required": ["columns"]
        }),
        renderer: Box::new(|node| {
            let p = &node.props;
            let columns = p
                .get("columns")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let header_cells = columns
                .iter()
                .map(|col| {
                    let label = col
                        .get("label")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    format!("<th>{label}</th>")
                })
                .collect::<Vec<_>>()
                .join("");
            let _data_source = str_prop(p, "data_source", "");
            let _pagination = bool_prop(p, "pagination", false);
            format!(
                r#"<table class="lc-table"><thead><tr>{header_cells}</tr></thead><tbody></tbody></table>"#
            )
        }),
    });

    // ---- form ----
    reg.register(ComponentEntry {
        type_key: "form".into(),
        category: "layout".into(),
        props_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "action": { "type": "string" },
                "method": { "type": "string", "enum": ["GET", "POST"] },
                "fields": { "type": "array" }
            },
            "required": []
        }),
        renderer: Box::new(|node| {
            let p = &node.props;
            let action = str_prop(p, "action", "");
            let method = str_prop(p, "method", "POST");
            let children_html = node
                .children
                .iter()
                .map(|child| format!(r#"<div class="lc-field">{}</div>"#, child.props))
                .collect::<Vec<_>>()
                .join("\n");
            format!(r#"<form action="{action}" method="{method}">{children_html}</form>"#)
        }),
    });

    // ---- image ----
    reg.register(ComponentEntry {
        type_key: "image".into(),
        category: "media".into(),
        props_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "src":       { "type": "string" },
                "alt":       { "type": "string" },
                "object_fit": { "type": "string", "enum": ["cover", "contain", "fill"] }
            },
            "required": ["src"]
        }),
        renderer: Box::new(|node| {
            let p = &node.props;
            let src = str_prop(p, "src", "");
            let alt = str_prop(p, "alt", "");
            let object_fit = str_prop(p, "object_fit", "cover");
            format!(r#"<img src="{src}" alt="{alt}" style="object-fit:{object_fit}" />"#)
        }),
    });

    // ---- divider ----
    reg.register(ComponentEntry {
        type_key: "divider".into(),
        category: "basic".into(),
        props_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "orientation": { "type": "string", "enum": ["horizontal", "vertical"], "default": "horizontal" }
            },
            "required": []
        }),
        renderer: Box::new(|node| {
            let orientation = str_prop(&node.props, "orientation", "horizontal");
            format!(
                r#"<hr class="lc-divider lc-divider--{orientation}" />"#
            )
        }),
    });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(type_key: &str, props: serde_json::Value) -> ComponentNode {
        ComponentNode {
            id: "test-node".into(),
            type_key: type_key.into(),
            props,
            grid_area: crate::schema::GridArea {
                col_start: 1,
                col_end: 2,
                row_start: 1,
                row_end: 2,
            },
            children: vec![],
        }
    }

    #[test]
    fn test_with_builtins_has_8_components() {
        let reg = ComponentRegistry::with_builtins();
        assert_eq!(reg.list().len(), 8);
    }

    #[test]
    fn test_all_builtins_have_renderer() {
        let reg = ComponentRegistry::with_builtins();
        for entry in reg.list() {
            let node = make_node(&entry.type_key, serde_json::json!({}));
            // Should not panic — every built-in renderer handles empty props
            let _html = (entry.renderer)(&node);
        }
    }

    #[test]
    fn test_register_and_get() {
        let mut reg = ComponentRegistry::new();
        reg.register(ComponentEntry {
            type_key: "custom".into(),
            category: "test".into(),
            props_schema: serde_json::json!({}),
            renderer: Box::new(|_| "<custom/>".into()),
        });
        assert!(reg.get("custom").is_some());
        assert_eq!(reg.get("custom").unwrap().category, "test");
    }

    #[test]
    fn test_unregister() {
        let mut reg = ComponentRegistry::with_builtins();
        assert!(reg.unregister("button"));
        assert!(!reg.unregister("button"));
        assert!(reg.get("button").is_none());
    }

    #[test]
    fn test_list_by_category() {
        let reg = ComponentRegistry::with_builtins();
        let basics = reg.list_by_category("basic");
        // button, input, text, divider = 4 basic components
        assert_eq!(basics.len(), 4);
        let layouts = reg.list_by_category("layout");
        assert_eq!(layouts.len(), 2); // container, form
    }

    #[test]
    fn test_validate_props_success() {
        let reg = ComponentRegistry::with_builtins();
        let props = serde_json::json!({ "label": "Click me" });
        assert!(reg.validate_props("button", &props).is_ok());
    }

    #[test]
    fn test_validate_props_missing_required() {
        let reg = ComponentRegistry::with_builtins();
        let props = serde_json::json!({});
        let result = reg.validate_props("button", &props);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("label")));
    }

    #[test]
    fn test_validate_props_wrong_type() {
        let reg = ComponentRegistry::with_builtins();
        let props = serde_json::json!({ "label": 123 });
        let result = reg.validate_props("button", &props);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("expected string")));
    }

    #[test]
    fn test_validate_props_enum() {
        let reg = ComponentRegistry::with_builtins();
        let props = serde_json::json!({ "label": "OK", "variant": "invalid" });
        let result = reg.validate_props("button", &props);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("not in enum")));
    }

    #[test]
    fn test_validate_unknown_type() {
        let reg = ComponentRegistry::with_builtins();
        let result = reg.validate_props("nonexistent", &serde_json::json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn test_render_button() {
        let reg = ComponentRegistry::with_builtins();
        let node = make_node("button", serde_json::json!({ "label": "Go" }));
        let html = reg.render(&node).unwrap();
        assert!(html.contains("<button"));
        assert!(html.contains("Go"));
        assert!(html.contains("lc-button--primary"));
    }

    #[test]
    fn test_render_button_disabled() {
        let reg = ComponentRegistry::with_builtins();
        let node = make_node(
            "button",
            serde_json::json!({ "label": "Off", "disabled": true }),
        );
        let html = reg.render(&node).unwrap();
        assert!(html.contains("disabled"));
    }

    #[test]
    fn test_render_text() {
        let reg = ComponentRegistry::with_builtins();
        let node = make_node(
            "text",
            serde_json::json!({ "content": "Hello", "tag": "h2", "align": "center" }),
        );
        let html = reg.render(&node).unwrap();
        assert!(html.contains("<h2"));
        assert!(html.contains("Hello"));
        assert!(html.contains("text-align:center"));
    }

    #[test]
    fn test_render_container_with_children() {
        let reg = ComponentRegistry::with_builtins();
        let child = ComponentNode {
            id: "c1".into(),
            type_key: "text".into(),
            props: serde_json::json!({ "content": "inner" }),
            grid_area: crate::schema::GridArea {
                col_start: 1,
                col_end: 2,
                row_start: 1,
                row_end: 2,
            },
            children: vec![],
        };
        let node = ComponentNode {
            id: "root".into(),
            type_key: "container".into(),
            props: serde_json::json!({ "direction": "row", "padding": "8px" }),
            grid_area: crate::schema::GridArea {
                col_start: 1,
                col_end: 4,
                row_start: 1,
                row_end: 4,
            },
            children: vec![child],
        };
        let html = reg.render(&node).unwrap();
        assert!(html.contains("flex-direction:row"));
        assert!(html.contains("padding:8px"));
        assert!(html.contains("lc-child"));
    }

    #[test]
    fn test_render_image() {
        let reg = ComponentRegistry::with_builtins();
        let node = make_node(
            "image",
            serde_json::json!({ "src": "/logo.png", "alt": "Logo" }),
        );
        let html = reg.render(&node).unwrap();
        assert!(html.contains(r#"src="/logo.png""#));
        assert!(html.contains(r#"alt="Logo""#));
    }

    #[test]
    fn test_render_divider() {
        let reg = ComponentRegistry::with_builtins();
        let node = make_node("divider", serde_json::json!({}));
        let html = reg.render(&node).unwrap();
        assert!(html.contains("<hr"));
        assert!(html.contains("horizontal"));
    }

    #[test]
    fn test_render_unknown_type() {
        let reg = ComponentRegistry::with_builtins();
        let node = make_node("nonexistent", serde_json::json!({}));
        assert!(reg.render(&node).is_err());
    }

    #[test]
    fn test_categories() {
        let reg = ComponentRegistry::with_builtins();
        let cats = reg.categories();
        assert!(cats.contains(&"basic".into()));
        assert!(cats.contains(&"layout".into()));
        assert!(cats.contains(&"data".into()));
        assert!(cats.contains(&"media".into()));
    }

    #[test]
    fn test_register_from_record() {
        let mut reg = ComponentRegistry::new();
        let record = ComponentDefRecord {
            id: uuid::Uuid::nil(),
            type_key: "my_widget".into(),
            props_schema: serde_json::json!({ "properties": {} }),
            category: "custom".into(),
            icon: None,
            created_at: "2025-01-01T00:00:00Z".into(),
        };
        reg.register_from_record(record, Box::new(|_| "<widget/>".into()));
        assert!(reg.get("my_widget").is_some());
        assert_eq!(reg.get("my_widget").unwrap().category, "custom");
    }
}
