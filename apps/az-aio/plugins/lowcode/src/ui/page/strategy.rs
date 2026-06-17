use std::sync::Arc;

use anyhow::Context;
use crate::contract::{
    DynLowcodeLayoutStrategy, LowcodeFieldDescriptor, LowcodeLayoutDescriptor,
    LowcodeLayoutOption, LowcodeLayoutStrategy, LowcodeModelDescriptor, LowcodeRelationDescriptor,
    LowcodeRenderContext, LowcodeScreenDescriptor, layout_descriptors,
};
use az_dioxus_components::workbench::{PageHeader, WorkbenchPage};
use dioxus::prelude::*;
use rudi::Singleton;

use crate::{
    backend::model::{
        AppScreen, FormConfig, FormField, MasterDetailConfig, MetaFieldView, TableColumn,
        TableConfig,
    },
    ui::page::renderers::{
        accordion::render_accordion, form::render_form, master_detail::render_master_detail,
        table::render_table_screen, tree_table::render_tree_table,
    },
};

pub fn layout_strategies() -> Vec<DynLowcodeLayoutStrategy> {
    let mut di = rudi::Context::auto_register();
    let mut strategies = di.resolve_by_type::<DynLowcodeLayoutStrategy>();
    if strategies.is_empty() {
        strategies = builtin_layout_strategies();
    }
    strategies.sort_by(|left, right| {
        let left = left.descriptor();
        let right = right.descriptor();
        left.order.cmp(&right.order).then(left.code.cmp(&right.code))
    });
    strategies
}

pub fn available_layouts() -> Vec<LowcodeLayoutDescriptor> {
    layout_descriptors(&layout_strategies())
}

pub fn render_screen_with_strategy(
    screen: &AppScreen,
    fields: &[MetaFieldView],
    query: &str,
) -> Element {
    let context = render_context_from_screen(screen, fields, query);
    let Some(strategy) = layout_strategies()
        .into_iter()
        .find(|strategy| strategy.descriptor().code == screen.layout)
    else {
        let layout = screen.layout.clone();
        return rsx! {
            WorkbenchPage {
                PageHeader {
                    title: screen.label.clone(),
                    subtitle: format!("未支持的布局: {layout}"),
                }
            }
        };
    };

    strategy.render(context).unwrap_or_else(|error| {
        let message = error.to_string();
        rsx! {
            WorkbenchPage {
                PageHeader {
                    title: screen.label.clone(),
                    subtitle: message,
                }
            }
        }
    })
}

pub fn auto_config_json_for_layout(layout: &str, fields: &[MetaFieldView]) -> String {
    let model = model_descriptor("scratch", "Scratch", "临时模型", fields);
    layout_strategies()
        .into_iter()
        .find(|strategy| strategy.descriptor().code == layout)
        .and_then(|strategy| strategy.default_config_json(&model).ok())
        .unwrap_or_else(|| "{}".to_string())
}

fn builtin_layout_strategies() -> Vec<DynLowcodeLayoutStrategy> {
    vec![
        Arc::new(TableLayoutStrategy),
        Arc::new(MasterDetailLayoutStrategy),
        Arc::new(AccordionLayoutStrategy),
        Arc::new(FormLayoutStrategy),
        Arc::new(TreeTableLayoutStrategy),
    ]
}

#[Singleton(name = "lowcode-table-layout")]
pub fn table_layout_strategy() -> DynLowcodeLayoutStrategy {
    Arc::new(TableLayoutStrategy)
}

#[Singleton(name = "lowcode-master-detail-layout")]
pub fn master_detail_layout_strategy() -> DynLowcodeLayoutStrategy {
    Arc::new(MasterDetailLayoutStrategy)
}

#[Singleton(name = "lowcode-accordion-layout")]
pub fn accordion_layout_strategy() -> DynLowcodeLayoutStrategy {
    Arc::new(AccordionLayoutStrategy)
}

#[Singleton(name = "lowcode-form-layout")]
pub fn form_layout_strategy() -> DynLowcodeLayoutStrategy {
    Arc::new(FormLayoutStrategy)
}

#[Singleton(name = "lowcode-tree-table-layout")]
pub fn tree_table_layout_strategy() -> DynLowcodeLayoutStrategy {
    Arc::new(TreeTableLayoutStrategy)
}

struct TableLayoutStrategy;
struct MasterDetailLayoutStrategy;
struct AccordionLayoutStrategy;
struct FormLayoutStrategy;
struct TreeTableLayoutStrategy;

impl LowcodeLayoutStrategy for TableLayoutStrategy {
    fn descriptor(&self) -> LowcodeLayoutDescriptor {
        layout_descriptor(
            "Table",
            "增删改查表格",
            "Filter bar, batch actions, sticky header, and configurable frozen columns.",
            10,
            vec![
                LowcodeLayoutOption::FilterBar,
                LowcodeLayoutOption::BatchActions,
                LowcodeLayoutOption::FrozenHeader,
                LowcodeLayoutOption::FrozenColumns,
                LowcodeLayoutOption::InlineForm,
            ],
        )
    }

    fn default_config_json(&self, model: &LowcodeModelDescriptor) -> anyhow::Result<String> {
        serde_json::to_string(&TableConfig {
            columns: non_relation_fields(model)
                .into_iter()
                .map(table_column_from_descriptor)
                .collect(),
            searchable_fields: first_text_field(model).into_iter().collect(),
            page_size: 20,
            frozen_header: true,
            frozen_columns: 1,
        })
        .context("serialize table lowcode config")
    }

    fn render(&self, context: LowcodeRenderContext) -> anyhow::Result<Element> {
        let fields = fields_from_descriptor(&context.model);
        Ok(render_table_screen(
            &context.screen.label,
            &context.screen.model_id,
            &fields,
            &context.screen.config_json,
            &context.query,
        ))
    }
}

impl LowcodeLayoutStrategy for MasterDetailLayoutStrategy {
    fn descriptor(&self) -> LowcodeLayoutDescriptor {
        layout_descriptor(
            "MasterDetail",
            "左树右表",
            "A left context tree filters the right-side dense table.",
            20,
            vec![
                LowcodeLayoutOption::LeftTree,
                LowcodeLayoutOption::FilterBar,
                LowcodeLayoutOption::FrozenHeader,
                LowcodeLayoutOption::FrozenColumns,
            ],
        )
    }

    fn default_config_json(&self, model: &LowcodeModelDescriptor) -> anyhow::Result<String> {
        let tree_field = model
            .fields
            .iter()
            .find(|field| {
                field
                    .relation
                    .as_ref()
                    .is_some_and(|relation| relation.relation_type == "SelfRecursive")
            })
            .map(|field| field.id.clone())
            .unwrap_or_default();
        serde_json::to_string(&MasterDetailConfig {
            tree_field_id: tree_field,
            detail_columns: non_relation_fields(model)
                .into_iter()
                .map(table_column_from_descriptor)
                .collect(),
            detail_searchable: first_text_field(model).into_iter().collect(),
        })
        .context("serialize master-detail lowcode config")
    }

    fn render(&self, context: LowcodeRenderContext) -> anyhow::Result<Element> {
        let fields = fields_from_descriptor(&context.model);
        Ok(render_master_detail(
            &context.screen.label,
            &context.screen.model_id,
            &fields,
            &context.screen.config_json,
            &context.query,
        ))
    }
}

impl LowcodeLayoutStrategy for AccordionLayoutStrategy {
    fn descriptor(&self) -> LowcodeLayoutDescriptor {
        layout_descriptor(
            "Accordion",
            "手风琴",
            "Grouped fields for high-density record detail and inline editing.",
            30,
            vec![LowcodeLayoutOption::AccordionGroups, LowcodeLayoutOption::InlineForm],
        )
    }

    fn render(&self, context: LowcodeRenderContext) -> anyhow::Result<Element> {
        let fields = fields_from_descriptor(&context.model);
        Ok(render_accordion(
            &context.screen.label,
            &context.screen.model_id,
            &fields,
        ))
    }
}

impl LowcodeLayoutStrategy for FormLayoutStrategy {
    fn descriptor(&self) -> LowcodeLayoutDescriptor {
        layout_descriptor(
            "Form",
            "表单",
            "Metadata-driven data entry form.",
            40,
            vec![LowcodeLayoutOption::InlineForm],
        )
    }

    fn default_config_json(&self, model: &LowcodeModelDescriptor) -> anyhow::Result<String> {
        serde_json::to_string(&FormConfig {
            fields: non_relation_fields(model)
                .into_iter()
                .map(|field| FormField {
                    field_name: field.name.clone(),
                    label: field.label.clone(),
                    field_type: field.field_type.clone(),
                    required: field.required,
                    placeholder: format!("输入{}", field.label),
                    options: field.enum_options.clone(),
                })
                .collect(),
            submit_label: "保存".to_string(),
        })
        .context("serialize form lowcode config")
    }

    fn render(&self, context: LowcodeRenderContext) -> anyhow::Result<Element> {
        let fields = fields_from_descriptor(&context.model);
        Ok(render_form(
            &context.screen.label,
            &context.screen.model_id,
            &fields,
            &context.screen.config_json,
        ))
    }
}

impl LowcodeLayoutStrategy for TreeTableLayoutStrategy {
    fn descriptor(&self) -> LowcodeLayoutDescriptor {
        layout_descriptor(
            "TreeTable",
            "树形表格",
            "Hierarchical table rendered from self-recursive relation metadata.",
            50,
            vec![
                LowcodeLayoutOption::FrozenHeader,
                LowcodeLayoutOption::FrozenColumns,
            ],
        )
    }

    fn render(&self, context: LowcodeRenderContext) -> anyhow::Result<Element> {
        let fields = fields_from_descriptor(&context.model);
        Ok(render_tree_table(
            &context.screen.label,
            &context.screen.model_id,
            &fields,
        ))
    }
}

fn render_context_from_screen(
    screen: &AppScreen,
    fields: &[MetaFieldView],
    query: &str,
) -> LowcodeRenderContext {
    LowcodeRenderContext {
        screen: LowcodeScreenDescriptor {
            id: screen.id.clone(),
            name: screen.name.clone(),
            label: screen.label.clone(),
            layout: screen.layout.clone(),
            model_id: screen.model_id.clone(),
            config_json: screen.config_json.clone(),
        },
        model: model_descriptor(&screen.model_id, &screen.name, &screen.label, fields),
        query: query.to_string(),
    }
}

fn model_descriptor(
    model_id: &str,
    name: &str,
    label: &str,
    fields: &[MetaFieldView],
) -> LowcodeModelDescriptor {
    LowcodeModelDescriptor {
        id: model_id.to_string(),
        name: name.to_string(),
        label: label.to_string(),
        description: String::new(),
        fields: fields
            .iter()
            .map(field_descriptor_from_view)
            .collect::<Vec<_>>(),
    }
}

fn field_descriptor_from_view(field: &MetaFieldView) -> LowcodeFieldDescriptor {
    LowcodeFieldDescriptor {
        id: field.id.clone(),
        name: field.name.clone(),
        label: field.label.clone(),
        field_type: field.field_type.clone(),
        order: field.order,
        required: field.is_required,
        unique: field.is_unique,
        relation: match (&field.relation_type, &field.relation_model_id) {
            (Some(relation_type), Some(target_model_id)) => {
                Some(LowcodeRelationDescriptor {
                    relation_type: relation_type.clone(),
                    target_model_id: target_model_id.clone(),
                })
            }
            _ => None,
        },
        default_value: field.default_value.clone(),
        enum_options: field
            .enum_options
            .as_deref()
            .unwrap_or_default()
            .split(',')
            .filter(|value| !value.trim().is_empty())
            .map(|value| value.trim().to_string())
            .collect(),
    }
}

fn fields_from_descriptor(model: &LowcodeModelDescriptor) -> Vec<MetaFieldView> {
    model
        .fields
        .iter()
        .map(|field| MetaFieldView {
            id: field.id.clone(),
            model_id: model.id.clone(),
            name: field.name.clone(),
            label: field.label.clone(),
            field_type: field.field_type.clone(),
            relation_type: field.relation.as_ref().map(|relation| relation.relation_type.clone()),
            relation_model_id: field
                .relation
                .as_ref()
                .map(|relation| relation.target_model_id.clone()),
            relation_model_name: None,
            is_required: field.required,
            is_unique: field.unique,
            order: field.order,
            default_value: field.default_value.clone(),
            enum_options: if field.enum_options.is_empty() {
                None
            } else {
                Some(field.enum_options.join(","))
            },
        })
        .collect()
}

fn layout_descriptor(
    code: &str,
    label: &str,
    description: &str,
    order: i32,
    supported_options: Vec<LowcodeLayoutOption>,
) -> LowcodeLayoutDescriptor {
    LowcodeLayoutDescriptor {
        code: code.to_string(),
        label: label.to_string(),
        description: description.to_string(),
        order,
        supported_options,
    }
}

fn non_relation_fields(model: &LowcodeModelDescriptor) -> Vec<&LowcodeFieldDescriptor> {
    model
        .fields
        .iter()
        .filter(|field| field.field_type != "Relation")
        .collect()
}

fn table_column_from_descriptor(field: &LowcodeFieldDescriptor) -> TableColumn {
    TableColumn {
        field_name: field.name.clone(),
        label: field.label.clone(),
        sortable: false,
        width: None,
    }
}

fn first_text_field(model: &LowcodeModelDescriptor) -> Option<String> {
    model
        .fields
        .iter()
        .find(|field| field.field_type == "String")
        .map(|field| field.name.clone())
}

#[cfg(test)]
mod tests {
    use crate::backend::model::MetaFieldView;

    use super::*;

    #[test]
    fn strategy_collection_exposes_configurable_layouts() {
        let labels = available_layouts()
            .into_iter()
            .map(|layout| layout.label)
            .collect::<Vec<_>>();

        assert_eq!(
            labels,
            vec!["增删改查表格", "左树右表", "手风琴", "表单", "树形表格"]
        );
    }

    #[test]
    fn table_strategy_default_config_enables_frozen_header_and_first_column() {
        let fields = vec![MetaFieldView {
            id: "name".to_string(),
            model_id: "m1".to_string(),
            name: "name".to_string(),
            label: "名称".to_string(),
            field_type: "String".to_string(),
            relation_type: None,
            relation_model_id: None,
            relation_model_name: None,
            is_required: false,
            is_unique: false,
            order: 1,
            default_value: None,
            enum_options: None,
        }];

        let json = auto_config_json_for_layout("Table", &fields);
        let config = serde_json::from_str::<TableConfig>(&json).unwrap();

        assert!(config.frozen_header);
        assert_eq!(config.frozen_columns, 1);
    }
}
