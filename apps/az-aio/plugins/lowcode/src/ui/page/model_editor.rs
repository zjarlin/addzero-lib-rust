use az_dioxus_components::{
    accordion::Accordion,
    status_badge::{StatusBadge, StatusBadgeTone},
    toolbar_button::{ToolbarButton, ToolbarButtonLink, ToolbarButtonTone},
    form::{ActionForm, CheckboxRow, FormRow, HiddenInput, Input, Select, SelectOption},
    table::{Table, TableBody, TableCell, TableHead, TableHeaderCell, TableRow},
    workbench::{
        PageHeader, SplitWorkbench, TableViewport, WorkbenchDetail,
        WorkbenchDetailHeader, WorkbenchPage, WorkbenchTree, WorkbenchTreeHeader,
        WorkbenchTreeList,
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
        WorkbenchPage {
            PageHeader {
                title: "低代码工作台",
                subtitle: "元数据建模 · 低代码页面管理",
            }
            SplitWorkbench {
                WorkbenchTree {
                    WorkbenchTreeHeader { title: "数据模型",
                        ToolbarButtonLink {
                            href: format!("/?route={lowcode_route}&mode=screens"),
                            class: "toolbar-button--compact",
                            "页面列表 →"
                        }
                    }
                    ActionForm {
                        div { style: "padding: 6px 8px 0;",
                            HiddenInput { name: "route", value: lowcode_route }
                            Input {
                                name: "search",
                                placeholder: "搜索模型...",
                                value: search.clone(),
                                class: "form-input--compact",
                            }
                        }
                    }
                    Accordion {
                        title: "＋ 新建模型",
                        class: "accordion--tree-form",
                        summary_class: "accordion__summary--compact",
                        body_class: "accordion__body--compact",
                        LowcodeActionForm { action_name: "new-model",
                            FormRow { label: "名称 · 英文标识", required: true,
                                Input { name: "name", placeholder: "Product", required: true, class: "form-input--compact" }
                            }
                            FormRow { label: "标签 · 中文显示", required: true,
                                Input { name: "label", placeholder: "产品", required: true, class: "form-input--compact" }
                            }
                            FormRow { label: "描述",
                                Input { name: "desc", placeholder: "用途说明", class: "form-input--compact" }
                            }
                            ToolbarButton { tone: ToolbarButtonTone::Primary, button_type: "submit", class: "toolbar-button--compact", "创建模型" }
                        }
                    }
                    WorkbenchTreeList { class: "workbench-tree__list--tight",
                        if models.is_empty() {
                            p { class: "platform-muted", style: "padding: 8px; font-size: 11px;", "暂无模型" }
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
                WorkbenchDetail {
                    if let Some(model) = selected_model {
                        WorkbenchDetailHeader {
                            title: format!("{} · 字段", model.label),
                            subtitle: format!("{} — {}", model.name, model.description),
                            div { class: "toolbar",
                                Accordion { title: "＋ 添加字段",
                                    LowcodeActionForm {
                                        action_name: "new-field",
                                        hidden_fields: vec![("model".to_string(), model.id.clone())],
                                        FormRow { label: "字段名 · 英文", required: true,
                                            Input { name: "field_name", placeholder: "price", required: true }
                                        }
                                        FormRow { label: "标签 · 中文", required: true,
                                            Input { name: "field_label", placeholder: "价格", required: true }
                                        }
                                        FormRow { label: "类型",
                                            Select { name: "field_type", options: field_type_options(None) }
                                        }
                                        FormRow { label: "关联类型",
                                            Select { name: "rel_type", options: relation_type_options(None) }
                                        }
                                        FormRow { label: "关联模型",
                                            Select { name: "rel_model_id", options: model_options(&all_models, None) }
                                        }
                                        FormRow { label: "默认值",
                                            Input { name: "def_val", placeholder: "可选", class: "form-input--compact" }
                                        }
                                        div { class: "checkbox-group",
                                            CheckboxRow { name: "is_req", label: "必填" }
                                            CheckboxRow { name: "is_uniq", label: "唯一" }
                                        }
                                        ToolbarButton { tone: ToolbarButtonTone::Primary, button_type: "submit", "添加字段" }
                                    }
                                }
                                ToolbarButtonLink {
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
                        WorkbenchDetailHeader {
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
        TableViewport {
            Table { bordered: true, dense: true,
                FieldTableHead {}
                TableBody {
                    if props.field_views.is_empty() {
                        TableRow {
                            TableCell { class: "table-view__cell--empty", colspan: 8,
                                "暂无字段 — 点击「添加字段」创建"
                            }
                        }
                    } else {
                        for (index, field) in props.field_views.iter().enumerate() {
                            TableRow {
                                TableCell { "{index + 1}" }
                                TableCell { code { "{field.name}" } }
                                TableCell { "{field.label}" }
                                TableCell {
                                    StatusBadge { "{ft_label(&field.field_type)}" }
                                }
                                TableCell {
                                    if field.field_type == "Relation" {
                                        StatusBadge { tone: StatusBadgeTone::Accent, "{rel_label(field.relation_type.as_deref())}" }
                                        if let Some(ref relation_model_name) = field.relation_model_name {
                                            span { " → {relation_model_name}" }
                                        }
                                    } else {
                                        "—"
                                    }
                                }
                                TableCell {
                                    if field.is_required {
                                        StatusBadge { tone: StatusBadgeTone::Warn, "必填" }
                                    }
                                }
                                TableCell {
                                    if field.is_unique {
                                        StatusBadge { "唯一" }
                                    }
                                }
                                TableCell {
                                    div { class: "row-actions",
                                        FieldEditor {
                                            all_models: props.all_models.clone(),
                                            field: field.clone(),
                                            lowcode_route: props.lowcode_route.clone(),
                                            model_id: props.model_id.clone(),
                                        }
                                        ToolbarButtonLink {
                                            href: format!(
                                                "/?route={}&model={}&action=delete-field&field_id={}",
                                                props.lowcode_route,
                                                props.model_id,
                                                field.id,
                                            ),
                                            tone: ToolbarButtonTone::Danger,
                                            class: "toolbar-button--compact",
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
        TableHead {
            TableRow {
                TableHeaderCell { "#" }
                TableHeaderCell { "字段名" }
                TableHeaderCell { "标签" }
                TableHeaderCell { "类型" }
                TableHeaderCell { "关联" }
                TableHeaderCell { "必填" }
                TableHeaderCell { "唯一" }
                TableHeaderCell { "操作" }
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
        Accordion {
            title: "编辑",
            class: "accordion--inline",
            summary_class: "accordion__summary--compact",
            LowcodeActionForm {
                action_name: "edit-field",
                route: props.lowcode_route.clone(),
                hidden_fields: vec![
                    ("field_id".to_string(), props.field.id.clone()),
                    ("model".to_string(), props.model_id.clone()),
                ],
                FormRow { label: "标签",
                    Input { name: "field_label", value: props.field.label.clone(), class: "form-input--compact" }
                }
                FormRow { label: "类型",
                    Select {
                        name: "field_type",
                        options: field_type_options(Some(props.field.field_type.clone())),
                    }
                }
                FormRow { label: "关联类型",
                    Select {
                        name: "rel_type",
                        options: relation_type_options(props.field.relation_type.clone()),
                    }
                }
                FormRow { label: "关联模型",
                    Select {
                        name: "rel_model_id",
                        options: model_options(&props.all_models, props.field.relation_model_id.clone()),
                    }
                }
                FormRow { label: "默认值",
                    Input { name: "def_val", value: default_value, placeholder: "可选", class: "form-input--compact" }
                }
                div { class: "checkbox-group",
                    CheckboxRow { name: "is_req", label: "必填", checked: props.field.is_required }
                    CheckboxRow { name: "is_uniq", label: "唯一", checked: props.field.is_unique }
                }
                ToolbarButton { tone: ToolbarButtonTone::Primary, button_type: "submit", class: "toolbar-button--compact", "保存" }
            }
        }
    }
}

#[allow(non_snake_case)]
fn EmptyFieldTable() -> Element {
    rsx! {
        TableViewport {
            Table { bordered: true, dense: true,
                FieldTableHead {}
                TableBody {
                    TableRow {
                        TableCell { class: "table-view__cell--empty", colspan: 8,
                            "← 选择左侧模型以查看字段 · 支持字符串、整数、关联、自递归树等类型"
                        }
                    }
                }
            }
        }
    }
}

fn field_type_options(current: Option<String>) -> Vec<SelectOption> {
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
        SelectOption::new(value, label).selected(current.as_deref() == Some(value))
    })
    .collect()
}

fn relation_type_options(current: Option<String>) -> Vec<SelectOption> {
    [
        ("", "—"),
        ("OneToOne", "一对一"),
        ("OneToMany", "一对多"),
        ("ManyToMany", "多对多"),
        ("SelfRecursive", "自递归 · 树"),
    ]
    .into_iter()
    .map(|(value, label)| {
        SelectOption::new(value, label).selected(current.as_deref() == Some(value))
    })
    .collect()
}

fn model_options(models: &[MetaModelSummary], current: Option<String>) -> Vec<SelectOption> {
    std::iter::once(SelectOption::new("", "—"))
        .chain(models.iter().map(|model| {
            SelectOption::new(model.id.clone(), format!("{} ({})", model.label, model.name))
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
