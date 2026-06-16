//! Runtime low-code contracts owned by the lowcode plugin.

use dioxus::prelude::Element;
use serde::{Deserialize, Serialize};

/// Shared layout strategy object used by the lowcode runtime.
pub type DynLowcodeLayoutStrategy = std::sync::Arc<dyn LowcodeLayoutStrategy>;

/// Shared metadata provider object used by the lowcode runtime.
pub type DynLowcodeMetadataProvider = std::sync::Arc<dyn LowcodeMetadataProvider>;

/// Configurable menu node contributed by the lowcode plugin itself.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct LowcodeMenuContribution {
    /// Stable menu id.
    pub id: String,
    /// Parent menu id when this node is nested.
    pub parent_id: Option<String>,
    /// Display label.
    pub label: String,
    /// Target route, optionally including query parameters.
    pub route: String,
    /// Display icon text.
    pub icon: String,
    /// Sort order inside the same parent.
    pub order: i32,
    /// Whether this menu should be visible.
    pub visible: bool,
    /// Permission keys where any matching permission can reveal the menu.
    #[serde(default)]
    pub permissions_any_of: Vec<String>,
    /// Extension metadata for future plugin-level configuration.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Lowcode model metadata used by generated screens.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct LowcodeModelDescriptor {
    /// Stable model id.
    pub id: String,
    /// Technical model name.
    pub name: String,
    /// Display label.
    pub label: String,
    /// Human readable description.
    pub description: String,
    /// Model field descriptors.
    #[serde(default)]
    pub fields: Vec<LowcodeFieldDescriptor>,
}

/// Lowcode field metadata used by generated forms and tables.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct LowcodeFieldDescriptor {
    /// Stable field id.
    pub id: String,
    /// Technical field name.
    pub name: String,
    /// Display label.
    pub label: String,
    /// Field type code.
    pub field_type: String,
    /// Sort order in forms and tables.
    pub order: i32,
    /// Whether this field must be provided.
    pub required: bool,
    /// Whether this field should be unique.
    pub unique: bool,
    /// Relation metadata when the field points to another model.
    pub relation: Option<LowcodeRelationDescriptor>,
    /// Default string value.
    pub default_value: Option<String>,
    /// Enum options for select-like renderers.
    #[serde(default)]
    pub enum_options: Vec<String>,
}

/// Relation metadata for a lowcode field.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LowcodeRelationDescriptor {
    /// Relation type code.
    pub relation_type: String,
    /// Target model id.
    pub target_model_id: String,
}

/// Layout strategy metadata exposed to screen configuration UI.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct LowcodeLayoutDescriptor {
    /// Stable layout code.
    pub code: String,
    /// Display label.
    pub label: String,
    /// Human readable description.
    pub description: String,
    /// Sort order in layout selectors.
    pub order: i32,
    /// Supported configuration options.
    #[serde(default)]
    pub supported_options: Vec<LowcodeLayoutOption>,
}

/// Capabilities supported by a lowcode layout strategy.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LowcodeLayoutOption {
    /// Renders a filter bar.
    FilterBar,
    /// Renders batch actions.
    BatchActions,
    /// Supports sticky table headers.
    FrozenHeader,
    /// Supports frozen leading columns.
    FrozenColumns,
    /// Supports a left tree with right detail/table area.
    LeftTree,
    /// Supports accordion grouping.
    AccordionGroups,
    /// Supports inline form editing.
    InlineForm,
}

/// Persisted lowcode screen descriptor used by runtime rendering.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LowcodeScreenDescriptor {
    /// Stable screen id.
    pub id: String,
    /// Technical screen name.
    pub name: String,
    /// Display label.
    pub label: String,
    /// Selected layout code.
    pub layout: String,
    /// Bound model id.
    pub model_id: String,
    /// Layout configuration JSON.
    pub config_json: String,
}

/// Context passed into a lowcode layout strategy during rendering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LowcodeRenderContext {
    /// Active screen metadata.
    pub screen: LowcodeScreenDescriptor,
    /// Active model metadata.
    pub model: LowcodeModelDescriptor,
    /// Current request query string.
    pub query: String,
}

/// Metadata provider abstraction collected by `rudi` inside the lowcode plugin.
pub trait LowcodeMetadataProvider: Send + Sync {
    /// Returns model descriptors currently available to the lowcode runtime.
    fn models(&self) -> anyhow::Result<Vec<LowcodeModelDescriptor>>;

    /// Returns configurable lowcode menu nodes.
    fn menus(&self) -> anyhow::Result<Vec<LowcodeMenuContribution>> {
        Ok(Vec::new())
    }
}

/// Strategy abstraction used to render one lowcode screen layout.
pub trait LowcodeLayoutStrategy: Send + Sync {
    /// Returns strategy metadata for configuration UI.
    fn descriptor(&self) -> LowcodeLayoutDescriptor;

    /// Returns default layout configuration JSON for a model.
    fn default_config_json(&self, _model: &LowcodeModelDescriptor) -> anyhow::Result<String> {
        Ok("{}".to_string())
    }

    /// Renders this strategy for the given context.
    fn render(&self, context: LowcodeRenderContext) -> anyhow::Result<Element>;
}

/// Sort layout descriptors from strategy objects.
pub fn layout_descriptors(
    strategies: &[DynLowcodeLayoutStrategy],
) -> Vec<LowcodeLayoutDescriptor> {
    let mut descriptors = strategies
        .iter()
        .map(|strategy| strategy.descriptor())
        .collect::<Vec<_>>();
    descriptors.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then(left.label.cmp(&right.label))
            .then(left.code.cmp(&right.code))
    });
    descriptors
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use dioxus::prelude::*;

    use super::*;

    struct TestStrategy(&'static str, i32);

    impl LowcodeLayoutStrategy for TestStrategy {
        fn descriptor(&self) -> LowcodeLayoutDescriptor {
            LowcodeLayoutDescriptor {
                code: self.0.to_string(),
                label: self.0.to_string(),
                description: String::new(),
                order: self.1,
                supported_options: Vec::new(),
            }
        }

        fn render(&self, _context: LowcodeRenderContext) -> anyhow::Result<Element> {
            Ok(rsx! { div {} })
        }
    }

    #[test]
    fn descriptors_follow_strategy_order() {
        let strategies: Vec<DynLowcodeLayoutStrategy> = vec![
            Arc::new(TestStrategy("table", 20)),
            Arc::new(TestStrategy("tree", 10)),
        ];

        let codes = layout_descriptors(&strategies)
            .into_iter()
            .map(|descriptor| descriptor.code)
            .collect::<Vec<_>>();

        assert_eq!(codes, vec!["tree", "table"]);
    }
}
