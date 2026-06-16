use az_dioxus_components::{
    az_accordion::AzAccordion,
    az_badge::{AzBadge, AzBadgeTone},
    az_button::{AzButton, AzButtonLink, AzButtonTone},
    az_form::{AzActionForm, AzCheckboxRow, AzFormRow, AzHiddenInput, AzInput, AzSelect, AzSelectOption},
    az_table::{AzTable, AzTableBody, AzTableCell, AzTableHead, AzTableHeaderCell, AzTableRow},
    az_workbench::{
        AzPageHeader, AzSplitWorkbench, AzTableViewport, AzWorkbenchDetail,
        AzWorkbenchDetailHeader, AzWorkbenchPage, AzWorkbenchTree, AzWorkbenchTreeHeader,
        AzWorkbenchTreeList,
    },
};
use dioxus::prelude::*;

use crate::backend::model::{MetaFieldView, MetaModelSummary};
use crate::ui::page::helpers::{ft_label, get_store, rel_label, LowcodeActionForm};

pub fn render_model_editor(selected_model_id: Option<String>, query: &str) -> Element {
    let lowcode_route = "/lowcode";
    let store = get_store();
    let all_models = store.list_models_sync();
    let selected: Option<&str> = selected_model_id.as_deref();
    let field_views: Vec<MetaFieldView> = selected
        .map(|mid| store.list_fields_sync(mid))
        .unwrap_or_default();
    let selected_model = selected.and_then(|mid| all_models.iter().find(|m| m.id == mid));

    let search = parse_q(query, "search").unwrap_or_default();
    let models: Vec<_> = if search.is_empty() {
        all_models.iter().collect()
    } else {
        let query = search.to_lowercase();
        all_models
            .iter()
            .filter(|model| {
                model.label.to_lowercase().contains(&query)
                    || model.name.to_lowercase().contains(&query)
            })
            .collect()
    };

    rsx! {
        AzWorkbenchPage {
            AzPageHeader {
                title: "低代码工作台",
                subtitle: "元数据建模 · 低代码页面管理",
            }
            AzSplitWorkbench {
                AzWorkbenchTree {
                    AzWorkbenchTreeHeader { title: "数据模型",
                        AzButtonLink {
                            href: format!("/?route={lowcode_route}&mode=screens"),
                            class: "az-button--compact",
                            "页面列表 →"
                        }
                    }
                    AzActionForm {
                        div { style: "padding: 6px 8px 0;",
                            AzHiddenInput { name: "route", value: lowcode_route }
                            AzInput {
                                name: "search",
                                placeholder: "搜索模型...",
                                value: search.clone(),
                                class: "az-input--compact",
                            }
                        }
                    }
                    AzAccordion {
                        title: "＋ 新建模型",
                        class: "az-accordion--tree-form",
                        summary_class: "az-accordion__summary--compact",
                        body_class: "az-accordion__body--compact",
                        LowcodeActionForm { action_name: "new-model",
                            AzFormRow { label: "名称 · 英文标识", required: true,
                                AzInput { name: "name", placeholder: "Product", required: true, class: "az-input--compact" }
                            }
                            AzFormRow { label: "标签 · 中文显示", required: true,
                                AzInput { name: "label", placeholder: "产品", required: true, class: "az-input--compact" }
                            }
                            AzFormRow { label: "描述",
                                AzInput { name: "desc", placeholder: "用途说明", class: "az-input--compact" }
                            }
                            AzButton { tone: AzButtonTone::Primary, button_type: "submit", class: "az-button--compact", "创建模型" }
                        }
                    }
                    AzWorkbenchTreeList { class: "az-workbench-tree__list--tight",
                        if models.is_empty() {
                            p { class: "az-platform-muted", style: "padding: 8px; font-size: 11px;", "暂无模型" }
                        } else {
                            for model in &models {
                                a {
                                    class: if selected.is_some_and(|id| id == model.id.as_str()) { "nav-button nav-button--active" } else { "nav-button" },
                                    href: "/?route={lowcode_route}&model={model.id}",
                                    span { class: "nav-button__icon", "▣" }
                                    span { class: "nav-button__label", "{model.label}" }
                                    span { class: "nav-button__meta", "{model.field_count}" }
                                }
                            }
                        }
                    }
                }
                AzWorkbenchDetail {
                    if let Some(model) = selected_model {
                        AzWorkbenchDetailHeader {
                            title: format!("{} · 字段", model.label),
                            subtitle: format!("{} — {}", model.name, model.description),
                            div { class: "az-toolbar",
                                AzAccordion { title: "＋ 添加字段",
                                    LowcodeActionForm {
                                        action_name: "new-field",
                                        hidden_fields: vec![("model".to_string(), model.id.clone())],
                                        AzFormRow { label: "字段名 · 英文", required: true,
                                            AzInput { name: "field_name", placeholder: "price", required: true }
                                        }
                                        AzFormRow { label: "标签 · 中文", required: true,
                                            AzInput { name: "field_label", placeholder: "价格", required: true }
                                        }
                                        AzFormRow { label: "类型",
                                            AzSelect { name: "field_type", options: field_type_options(None) }
                                        }
                                        AzFormRow { label: "关联类型",
                                            AzSelect { name: "rel_type", options: relation_type_options(None) }
                                        }
                                        AzFormRow { label: "关联模型",
                                            AzSelect { name: "rel_model_id", options: model_options(&all_models, None) }
                                        }
                                        AzFormRow { label: "默认值",
                                            AzInput { name: "def_val", placeholder: "可选", class: "az-input--compact" }
                                        }
                                        div { class: "az-checkbox-group",
                                            AzCheckboxRow { name: "is_req", label: "必填" }
                                            AzCheckboxRow { name: "is_uniq", label: "唯一" }
                                        }
                                        AzButton { tone: AzButtonTone::Primary, button_type: "submit", "添加字段" }
                                    }
                                }
                                AzButtonLink {
                                    href: format!("/?route={lowcode_route}&mode=screens"),
                                    "页面管理 →"
                                }
                            }
                        }
                        FieldTable {
                            all_models: all_models.clone(),
                            field_views: field_views.clone(),
                            lowcode_route: lowcode_route.to_string(),
                            model_id: model.id.clone(),
                        }
                    } else {
                        AzWorkbenchDetailHeader {
                            title: "字段定义",
                            subtitle: "选择左侧模型查看和管理字段",
                        }
                        EmptyFieldTable {}
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct FieldTableProps {
    all_models: Vec<MetaModelSummary>,
    field_views: Vec<MetaFieldView>,
    lowcode_route: String,
    model_id: String,
}

#[allow(non_snake_case)]
fn FieldTable(props: FieldTableProps) -> Element {
    rsx! {
        AzTableViewport {
            AzTable { bordered: true, dense: true,
                FieldTableHead {}
                AzTableBody {
                    if props.field_views.is_empty() {
                        AzTableRow {
                            AzTableCell { class: "az-table__cell--empty", colspan: 8,
                                "暂无字段 — 点击「添加字段」创建"
                            }
                        }
                    } else {
                        for (index, field) in props.field_views.iter().enumerate() {
                            AzTableRow {
                                AzTableCell { "{index + 1}" }
                                AzTableCell { code { "{field.name}" } }
                                AzTableCell { "{field.label}" }
                                AzTableCell {
                                    AzBadge { "{ft_label(&field.field_type)}" }
                                }
                                AzTableCell {
                                    if field.field_type == "Relation" {
                                        AzBadge { tone: AzBadgeTone::Accent, "{rel_label(field.relation_type.as_deref())}" }
                                        if let Some(ref relation_model_name) = field.relation_model_name {
                                            span { " → {relation_model_name}" }
                                        }
                                    } else {
                                        "—"
                                    }
                                }
                                AzTableCell {
                                    if field.is_required {
                                        AzBadge { tone: AzBadgeTone::Warn, "必填" }
                                    }
                                }
                                AzTableCell {
                                    if field.is_unique {
                                        AzBadge { "唯一" }
                                    }
                                }
                                AzTableCell {
                                    div { class: "az-row-actions",
                                        FieldEditor {
                                            all_models: props.all_models.clone(),
                                            field: field.clone(),
                                            lowcode_route: props.lowcode_route.clone(),
                                            model_id: props.model_id.clone(),
                                        }
                                        AzButtonLink {
                                            href: format!(
                                                "/?route={}&model={}&action=delete-field&field_id={}",
                                                props.lowcode_route,
                                                props.model_id,
                                                field.id,
                                            ),
                                            tone: AzButtonTone::Danger,
                                            class: "az-button--compact",
                                            "删除"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn FieldTableHead() -> Element {
    rsx! {
        AzTableHead {
            AzTableRow {
                AzTableHeaderCell { "#" }
                AzTableHeaderCell { "字段名" }
                AzTableHeaderCell { "标签" }
                AzTableHeaderCell { "类型" }
                AzTableHeaderCell { "关联" }
                AzTableHeaderCell { "必填" }
                AzTableHeaderCell { "唯一" }
                AzTableHeaderCell { "操作" }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct FieldEditorProps {
    all_models: Vec<MetaModelSummary>,
    field: MetaFieldView,
    lowcode_route: String,
    model_id: String,
}

#[allow(non_snake_case)]
fn FieldEditor(props: FieldEditorProps) -> Element {
    let default_value = props.field.default_value.clone().unwrap_or_default();

    rsx! {
        AzAccordion {
            title: "编辑",
            class: "az-accordion--inline",
            summary_class: "az-accordion__summary--compact",
            LowcodeActionForm {
                action_name: "edit-field",
                route: props.lowcode_route.clone(),
                hidden_fields: vec![
                    ("field_id".to_string(), props.field.id.clone()),
                    ("model".to_string(), props.model_id.clone()),
                ],
                AzFormRow { label: "标签",
                    AzInput { name: "field_label", value: props.field.label.clone(), class: "az-input--compact" }
                }
                AzFormRow { label: "类型",
                    AzSelect {
                        name: "field_type",
                        options: field_type_options(Some(props.field.field_type.clone())),
                    }
                }
                AzFormRow { label: "关联类型",
                    AzSelect {
                        name: "rel_type",
                        options: relation_type_options(props.field.relation_type.clone()),
                    }
                }
                AzFormRow { label: "关联模型",
                    AzSelect {
                        name: "rel_model_id",
                        options: model_options(&props.all_models, props.field.relation_model_id.clone()),
                    }
                }
                AzFormRow { label: "默认值",
                    AzInput { name: "def_val", value: default_value, placeholder: "可选", class: "az-input--compact" }
                }
                div { class: "az-checkbox-group",
                    AzCheckboxRow { name: "is_req", label: "必填", checked: props.field.is_required }
                    AzCheckboxRow { name: "is_uniq", label: "唯一", checked: props.field.is_unique }
                }
                AzButton { tone: AzButtonTone::Primary, button_type: "submit", class: "az-button--compact", "保存" }
            }
        }
    }
}

#[allow(non_snake_case)]
fn EmptyFieldTable() -> Element {
    rsx! {
        AzTableViewport {
            AzTable { bordered: true, dense: true,
                FieldTableHead {}
                AzTableBody {
                    AzTableRow {
                        AzTableCell { class: "az-table__cell--empty", colspan: 8,
                            "← 选择左侧模型以查看字段 · 支持字符串、整数、关联、自递归树等类型"
                        }
                    }
                }
            }
        }
    }
}

fn field_type_options(current: Option<String>) -> Vec<AzSelectOption> {
    [
        ("String", "字符串"),
        ("Integer", "整数"),
        ("Float", "浮点数"),
        ("Boolean", "布尔"),
        ("DateTime", "日期时间"),
        ("Json", "JSON"),
        ("Relation", "关联"),
    ]
    .into_iter()
    .map(|(value, label)| {
        AzSelectOption::new(value, label).selected(current.as_deref() == Some(value))
    })
    .collect()
}

fn relation_type_options(current: Option<String>) -> Vec<AzSelectOption> {
    [
        ("", "—"),
        ("OneToOne", "一对一"),
        ("OneToMany", "一对多"),
        ("ManyToMany", "多对多"),
        ("SelfRecursive", "自递归 · 树"),
    ]
    .into_iter()
    .map(|(value, label)| {
        AzSelectOption::new(value, label).selected(current.as_deref() == Some(value))
    })
    .collect()
}

fn model_options(models: &[MetaModelSummary], current: Option<String>) -> Vec<AzSelectOption> {
    std::iter::once(AzSelectOption::new("", "—"))
        .chain(models.iter().map(|model| {
            AzSelectOption::new(model.id.clone(), format!("{} ({})", model.label, model.name))
                .selected(current.as_deref() == Some(model.id.as_str()))
        }))
        .collect()
}

fn parse_q(query: &str, key: &str) -> Option<String> {
    let qs = query.split('?').nth(1).unwrap_or(query);
    for pair in qs.split('&') {
        let mut parts = pair.splitn(2, '=');
        if parts.next()? == key {
            return parts
                .next()
                .map(|v| urlencoding::decode(v).unwrap_or_else(|_| v.into()).into());
        }
    }
    None
}
