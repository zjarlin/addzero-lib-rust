//! Component registry — runtime type registration, props validation, and rendering.
//!
//! Manages `ComponentEntry` objects that pair a JSON Schema definition with a
//! renderer closure. Ships with built-in component types including basic form
//! elements, layout containers, data table, media, and `az-edge`.

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::edge::{AzEdgeParamType, AzEdgeVariant};
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
                if let Some(name) = field.as_str()
                    && (props.get(name).is_none() || props.get(name).unwrap().is_null())
                {
                    errors.push(format!("missing required field: {name}"));
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
                    if let Some(enum_vals) = field_schema.get("enum").and_then(|v| v.as_array())
                        && !enum_vals.contains(value)
                    {
                        errors.push(format!(
                            "field '{field_name}': value {value} not in enum {:?}",
                            enum_vals,
                        ));
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
        // Validate props before rendering to enforce schema constraints.
        self.validate_props(&node.type_key, &node.props)
            .map_err(|errs| errs.join("; "))?;
        let entry = self
            .entries
            .get(&node.type_key)
            .ok_or_else(|| format!("unknown component type: {}", node.type_key))?;
        Ok((entry.renderer)(node))
    }

    /// Create a registry pre-loaded with the built-in component types.
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

/// Escape HTML special characters to prevent XSS.
fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Escape a value for use inside an HTML attribute.
fn escape_attr(input: &str) -> String {
    escape_html(input)
}

/// Register the built-in component types.
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
            let label = escape_html(&str_prop(p, "label", ""));
            let variant = {
                let raw = str_prop(p, "variant", "primary");
                match raw.as_str() {
                    "primary" | "secondary" | "danger" => raw,
                    _ => "primary".to_string(),
                }
            };
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
            let placeholder = escape_attr(&str_prop(p, "placeholder", ""));
            let input_type = {
                let raw = str_prop(p, "input_type", "text");
                match raw.as_str() {
                    "text" | "email" | "password" | "number" => raw,
                    _ => "text".to_string(),
                }
            };
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
            let content = escape_html(&str_prop(p, "content", ""));
            let tag = {
                let raw = str_prop(p, "tag", "p");
                match raw.as_str() {
                    "p" | "h1" | "h2" | "h3" | "h4" | "span" => raw,
                    _ => "p".to_string(),
                }
            };
            let align = escape_attr(&str_prop(p, "align", "left"));
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
            let direction = {
                let raw = str_prop(p, "direction", "column");
                match raw.as_str() {
                    "row" | "column" => raw,
                    _ => "column".to_string(),
                }
            };
            let padding = escape_attr(&str_prop(p, "padding", "0"));
            let children_html = node
                .children
                .iter()
                .map(|child| {
                    // Inline mini-render: produce a placeholder for children
                    // that aren't rendered through the full registry path.
                    format!(
                        r#"<div class="lc-child" data-id="{}">{}</div>"#,
                        escape_attr(&child.id),
                        escape_html(&child.props.to_string())
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
                    format!("<th>{}</th>", escape_html(label))
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

    // ---- az-edge ----
    reg.register(ComponentEntry {
        type_key: "az-edge".into(),
        category: "edge".into(),
        props_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "title":    { "type": "string" },
                "variant":  { "type": "string", "enum": ["curl", "python", "rhai", "ts"] },
                "method":   { "type": "string", "enum": ["GET", "POST", "PUT", "PATCH", "DELETE"] },
                "path":     { "type": "string" },
                "template": { "type": "string" },
                "inputs":   { "type": "array" },
                "outputs":  { "type": "array" },
                "timeout_secs": { "type": "integer" }
            },
            "required": ["title", "variant", "method", "path", "template"]
        }),
        renderer: Box::new(|node| render_az_edge_card(&node.props)),
    });
}

fn render_az_edge_card(props: &serde_json::Value) -> String {
    let title = str_prop(props, "title", "az-edge");
    let variant = parse_edge_variant(str_prop(props, "variant", "rhai").as_str());
    let method = str_prop(props, "method", "POST");
    let path = str_prop(props, "path", "/api/edge");
    let template = str_prop(props, "template", "");
    let timeout_secs = props
        .get("timeout_secs")
        .and_then(|value| value.as_u64())
        .map(|value| value.to_string())
        .unwrap_or_default();
    let inputs = parse_edge_params(props, "inputs");
    let outputs = parse_edge_params(props, "outputs");

    format!(
        r#"<section class="lc-az-edge-card lc-az-edge-card--{variant}" data-runtime="{variant}">
<div class="lc-az-edge-card__summary">
  <span class="lc-az-edge-card__summary-title">az-edge</span>
  <span class="lc-az-edge-card__summary-route">{method} {path}</span>
</div>
<div class="lc-az-edge-card__form">
  <div class="lc-az-edge-card__header">
    <input class="lc-az-edge-card__title-input" name="title" value="{title}" placeholder="Node title" />
    <select class="lc-az-edge-card__runtime-select" name="variant">{runtime_options}</select>
  </div>
  <div class="lc-az-edge-card__meta">
    <label class="lc-az-edge-card__field">
      <span>Method</span>
      <select name="method">{method_options}</select>
    </label>
    <label class="lc-az-edge-card__field lc-az-edge-card__field--path">
      <span>Path</span>
      <input name="path" value="{path}" placeholder="/api/edge/name" />
    </label>
    <label class="lc-az-edge-card__field lc-az-edge-card__field--timeout">
      <span>Timeout</span>
      <input type="number" min="0" step="1" name="timeout_secs" value="{timeout_secs}" placeholder="0" />
    </label>
  </div>
  <label class="lc-az-edge-card__field lc-az-edge-card__field--template">
    <span>Template</span>
    <textarea name="template" spellcheck="false" placeholder="curl https://api.example.com?q={{{{city}}}}">{template}</textarea>
  </label>
  <div class="lc-az-edge-card__io">
    <section class="lc-az-edge-card__param-block" data-param-scope="inputs">
      <div class="lc-az-edge-card__param-header">
        <span>Inputs</span>
        <button type="button" class="lc-az-edge-card__param-add" data-add-row="inputs">Add</button>
      </div>
      <div class="lc-az-edge-card__param-list">{input_rows}</div>
    </section>
    <section class="lc-az-edge-card__param-block" data-param-scope="outputs">
      <div class="lc-az-edge-card__param-header">
        <span>Outputs</span>
        <button type="button" class="lc-az-edge-card__param-add" data-add-row="outputs">Add</button>
      </div>
      <div class="lc-az-edge-card__param-list">{output_rows}</div>
    </section>
  </div>
</div>
</section>"#,
        variant = variant.as_str(),
        title = escape_attr(&title),
        method = escape_html(&method),
        path = escape_html(&path),
        timeout_secs = escape_attr(&timeout_secs),
        template = escape_html(&template),
        runtime_options = render_runtime_options(variant),
        method_options = render_method_options(&method),
        input_rows = render_param_rows("inputs", &inputs),
        output_rows = render_param_rows("outputs", &outputs),
    )
}

#[derive(Clone)]
struct EdgeParamDraft {
    name: String,
    ty: AzEdgeParamType,
    required: bool,
    default_value: String,
    description: String,
}

impl EdgeParamDraft {
    fn blank() -> Self {
        Self {
            name: String::new(),
            ty: AzEdgeParamType::String,
            required: true,
            default_value: String::new(),
            description: String::new(),
        }
    }
}

fn parse_edge_variant(value: &str) -> AzEdgeVariant {
    match value {
        "curl" => AzEdgeVariant::Curl,
        "python" => AzEdgeVariant::Python,
        "ts" | "typescript" => AzEdgeVariant::TypeScript,
        _ => AzEdgeVariant::Rhai,
    }
}

fn parse_edge_param_type(value: &str) -> AzEdgeParamType {
    match value {
        "number" => AzEdgeParamType::Number,
        "boolean" => AzEdgeParamType::Boolean,
        "json" => AzEdgeParamType::Json,
        _ => AzEdgeParamType::String,
    }
}

fn parse_edge_params(props: &serde_json::Value, key: &str) -> Vec<EdgeParamDraft> {
    props
        .get(key)
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .map(|item| EdgeParamDraft {
                    name: item
                        .get("name")
                        .and_then(|value| value.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    ty: parse_edge_param_type(
                        item.get("type")
                            .and_then(|value| value.as_str())
                            .unwrap_or("string"),
                    ),
                    required: item
                        .get("required")
                        .and_then(|value| value.as_bool())
                        .unwrap_or(true),
                    default_value: item
                        .get("default_value")
                        .map(stringify_json_value)
                        .unwrap_or_default(),
                    description: item
                        .get("description")
                        .and_then(|value| value.as_str())
                        .unwrap_or_default()
                        .to_string(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn render_runtime_options(selected: AzEdgeVariant) -> String {
    [
        ("curl", "curl", selected == AzEdgeVariant::Curl),
        ("python", "python", selected == AzEdgeVariant::Python),
        ("rhai", "rhai", selected == AzEdgeVariant::Rhai),
        ("ts", "ts", selected == AzEdgeVariant::TypeScript),
    ]
    .into_iter()
    .map(|(value, label, is_selected)| {
        let selected_attr = if is_selected { " selected" } else { "" };
        format!(r#"<option value="{value}"{selected_attr}>{label}</option>"#)
    })
    .collect::<Vec<_>>()
    .join("")
}

fn render_method_options(selected: &str) -> String {
    ["GET", "POST", "PUT", "PATCH", "DELETE"]
        .into_iter()
        .map(|method| {
            let selected_attr = if method == selected { " selected" } else { "" };
            format!(r#"<option value="{method}"{selected_attr}>{method}</option>"#)
        })
        .collect::<Vec<_>>()
        .join("")
}

fn render_param_rows(scope: &str, params: &[EdgeParamDraft]) -> String {
    let rows = if params.is_empty() {
        vec![EdgeParamDraft::blank()]
    } else {
        params.to_vec()
    };

    rows.into_iter()
        .enumerate()
        .map(|(index, param)| render_param_row(scope, index, &param))
        .collect::<Vec<_>>()
        .join("")
}

fn render_param_row(scope: &str, index: usize, param: &EdgeParamDraft) -> String {
    let required_attr = if param.required { " checked" } else { "" };
    format!(
        r#"<div class="lc-az-edge-card__param-row" data-param-scope="{scope}">
  <div class="lc-az-edge-card__param-main">
    <input data-field="name" name="{scope}[{index}][name]" value="{name}" placeholder="name" />
    <select data-field="type" name="{scope}[{index}][type]">{type_options}</select>
    <input data-field="default" name="{scope}[{index}][default_value]" value="{default_value}" placeholder="default" />
    <label class="lc-az-edge-card__required">
      <input data-field="required" type="checkbox" name="{scope}[{index}][required]"{required_attr} />
      <span>required</span>
    </label>
    <button type="button" class="lc-az-edge-card__param-remove" data-remove-row>Remove</button>
  </div>
  <input class="lc-az-edge-card__param-description" data-field="description" name="{scope}[{index}][description]" value="{description}" placeholder="description" />
</div>"#,
        name = escape_attr(&param.name),
        default_value = escape_attr(&param.default_value),
        description = escape_attr(&param.description),
        type_options = render_param_type_options(param.ty),
    )
}

fn render_param_type_options(selected: AzEdgeParamType) -> String {
    [
        ("string", "string", selected == AzEdgeParamType::String),
        ("number", "number", selected == AzEdgeParamType::Number),
        ("boolean", "boolean", selected == AzEdgeParamType::Boolean),
        ("json", "json", selected == AzEdgeParamType::Json),
    ]
    .into_iter()
    .map(|(value, label, is_selected)| {
        let selected_attr = if is_selected { " selected" } else { "" };
        format!(r#"<option value="{value}"{selected_attr}>{label}</option>"#)
    })
    .collect::<Vec<_>>()
    .join("")
}

fn stringify_json_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Bool(value) => value.to_string(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => value.to_string(),
    }
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
    fn test_with_builtins_has_9_components() {
        let reg = ComponentRegistry::with_builtins();
        assert_eq!(reg.list().len(), 9);
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
        let edges = reg.list_by_category("edge");
        assert_eq!(edges.len(), 1); // az-edge
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
    fn test_render_az_edge_card() {
        let reg = ComponentRegistry::with_builtins();
        let node = make_node(
            "az-edge",
            serde_json::json!({
                "title": "Weather bridge",
                "variant": "curl",
                "method": "POST",
                "path": "/api/edge/weather",
                "template": "curl https://api.example.com/weather?q={{city}}",
                "inputs": [{ "name": "city", "type": "string" }],
                "outputs": [{ "name": "temperature", "type": "number" }]
            }),
        );
        let html = reg.render(&node).unwrap();
        assert!(html.contains("lc-az-edge-card"));
        assert!(html.contains("Weather bridge"));
        assert!(html.contains("data-runtime=\"curl\""));
        assert!(html.contains("/api/edge/weather"));
        assert!(html.contains("<textarea"));
        assert!(html.contains("name=\"template\""));
        assert!(html.contains("data-add-row=\"inputs\""));
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
        assert!(cats.contains(&"edge".into()));
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
