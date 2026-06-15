use dioxus::prelude::*;

use crate::backend::model::MetaFieldView;
use crate::ui::page::helpers::{ft_label, get_store, rel_label};

pub fn render_model_editor(selected_model_id: Option<String>, query: &str) -> Element {
    let lowcode_route = "/lowcode";
    let store = get_store();
    let all_models = store.list_models_sync();
    let selected: Option<&str> = selected_model_id.as_deref();
    let field_views: Vec<MetaFieldView> = selected
        .map(|mid| store.list_fields_sync(mid))
        .unwrap_or_default();
    let selected_model = selected.and_then(|mid| all_models.iter().find(|m| m.id == mid));

    // Read search query for model filtering
    let search = parse_q(query, "search").unwrap_or_default();
    let models: Vec<_> = if search.is_empty() {
        all_models.iter().collect()
    } else {
        let q = search.to_lowercase();
        all_models
            .iter()
            .filter(|m| m.label.to_lowercase().contains(&q) || m.name.to_lowercase().contains(&q))
            .collect()
    };

    rsx! {
        section { class: "lowcode-page",
            header { class: "lowcode-page__header",
                h1 { "低代码工作台" }
                p { "元数据建模 · 低代码页面管理" }
            }
            div { class: "lowcode-workbench",
                aside { class: "lowcode-tree",
                    div { class: "lowcode-tree__header",
                        h2 { "数据模型" }
                        a {
                            href: "/?route={lowcode_route}&mode=screens",
                            class: "toolbar-button",
                            style: "font-size: 11px; height: 24px; padding: 0 8px;",
                            "页面列表 →"
                        }
                    }
                    // Search box
                    form {
                        method: "get",
                        action: "/",
                        div { style: "padding: 6px 8px 0;",
                            input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                            input {
                                class: "settings-input",
                                name: "search",
                                placeholder: "搜索模型...",
                                value: "{search}",
                                style: "height: 28px; font-size: 12px;",
                            }
                        }
                    }
                    // New model form — compact, above list
                    details { class: "lowcode-accordion", style: "margin: 6px 8px; border: none; background: rgba(255,255,255,0.04);",
                        summary { class: "lowcode-accordion__summary", style: "font-size: 12px; padding: 5px 10px;", "＋ 新建模型" }
                        div { class: "lowcode-accordion__body", style: "padding: 8px 10px; gap: 6px;",
                            form {
                                method: "get",
                                action: "/",
                                input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                                input { r#type: "hidden", name: "action", value: "new-model" }
                                div { class: "settings-form-row",
                                    label { "名称 · 英文标识" }
                                    input { class: "settings-input", name: "name", placeholder: "Product", required: "required", style: "height: 28px; font-size: 12px;" }
                                }
                                div { class: "settings-form-row",
                                    label { "标签 · 中文显示" }
                                    input { class: "settings-input", name: "label", placeholder: "产品", required: "required", style: "height: 28px; font-size: 12px;" }
                                }
                                div { class: "settings-form-row",
                                    label { "描述" }
                                    input { class: "settings-input", name: "desc", placeholder: "用途说明", style: "height: 28px; font-size: 12px;" }
                                }
                                button { class: "toolbar-button toolbar-button--primary", r#type: "submit", style: "font-size: 11px; height: 26px;", "创建模型" }
                            }
                        }
                    }
                    // Model list — tight spacing
                    div { class: "lowcode-tree__list", style: "gap: 0;",
                        if models.is_empty() {
                            p { class: "az-platform-muted", style: "padding: 8px; font-size: 11px;", "暂无模型" }
                        } else {
                            for m in &models {
                                a {
                                    class: if selected.is_some_and(|s| s == m.id.as_str()) { "nav-button nav-button--active" } else { "nav-button" },
                                    style: "min-height: 24px; padding: 3px 8px; font-size: 12px; gap: 6px; border-radius: 0;",
                                    href: "/?route={lowcode_route}&model={m.id}",
                                    span { class: "nav-button__icon", style: "font-size: 13px;", "📦" }
                                    span { class: "nav-button__label", "{m.label}" }
                                    span { class: "nav-button__meta", style: "font-size: 10px; color: var(--text-subtle);", "{m.field_count}" }
                                }
                            }
                        }
                    }
                }
                section { class: "lowcode-detail", style: "overflow: hidden; display: grid; grid-template-rows: auto 1fr;",
                    if let Some(m) = selected_model {
                        div { class: "lowcode-detail__header",
                            h2 { "{m.label} · 字段" }
                            p { class: "lowcode-detail__subtitle", "{m.name} — {m.description}" }
                            div { style: "display: flex; gap: 8px; margin-top: 8px;",
                                details { class: "lowcode-accordion",
                                    summary { class: "lowcode-accordion__summary", "＋ 添加字段" }
                                    div { class: "lowcode-accordion__body",
                                        form {
                                            method: "get",
                                            action: "/",
                                            input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                                            input { r#type: "hidden", name: "action", value: "new-field" }
                                            input { r#type: "hidden", name: "model", value: "{m.id}" }
                                            div { class: "settings-form-row",
                                                label { "字段名 · 英文" }
                                                input { class: "settings-input", name: "field_name", placeholder: "price", required: "required" }
                                            }
                                            div { class: "settings-form-row",
                                                label { "标签 · 中文" }
                                                input { class: "settings-input", name: "field_label", placeholder: "价格", required: "required" }
                                            }
                                            div { class: "settings-form-row",
                                                label { "类型" }
                                                select { class: "settings-input", name: "field_type",
                                                    option { value: "String", "字符串" }
                                                    option { value: "Integer", "整数" }
                                                    option { value: "Float", "浮点数" }
                                                    option { value: "Boolean", "布尔" }
                                                    option { value: "DateTime", "日期时间" }
                                                    option { value: "Json", "JSON" }
                                                    option { value: "Relation", "关联" }
                                                }
                                            }
                                            div { class: "settings-form-row",
                                                label { "关联类型" }
                                                select { class: "settings-input", name: "rel_type",
                                                    option { value: "", "—" }
                                                    option { value: "OneToOne", "一对一" }
                                                    option { value: "OneToMany", "一对多" }
                                                    option { value: "ManyToMany", "多对多" }
                                                    option { value: "SelfRecursive", "自递归 · 树" }
                                                }
                                            }
                                            div { class: "settings-form-row",
                                                label { "关联模型" }
                                                select { class: "settings-input", name: "rel_model_id",
                                                    option { value: "", "—" }
                                                    for rm in &all_models {
                                                        option { value: "{rm.id}", "{rm.label} ({rm.name})" }
                                                    }
                                                }
                                            }
                                            div { class: "settings-form-row",
                                                label { "默认值" }
                                                input { class: "settings-input", name: "def_val", placeholder: "可选", style: "font-size: 12px;" }
                                            }
                                            div { class: "settings-form-row", style: "display: flex; gap: 16px; align-items: center; padding-top: 4px;",
                                                label { style: "display: flex; align-items: center; gap: 4px; font-size: 12px;",
                                                    input { r#type: "checkbox", name: "is_req", value: "1", style: "width: auto; height: auto;" }
                                                    "必填"
                                                }
                                                label { style: "display: flex; align-items: center; gap: 4px; font-size: 12px;",
                                                    input { r#type: "checkbox", name: "is_uniq", value: "1", style: "width: auto; height: auto;" }
                                                    "唯一"
                                                }
                                            }
                                            button { class: "toolbar-button toolbar-button--primary", r#type: "submit", "添加字段" }
                                        }
                                    }
                                }
                                a {
                                    href: "/?route={lowcode_route}&mode=screens",
                                    class: "toolbar-button",
                                    "页面管理 →"
                                }
                            }
                        }
                        div { class: "lowcode-table-scroll",
                            table { class: "az-table az-table--bordered az-table--dense",
                                thead {
                                    tr {
                                        th { class: "az-table__header-cell", "#" }
                                        th { class: "az-table__header-cell", "字段名" }
                                        th { class: "az-table__header-cell", "标签" }
                                        th { class: "az-table__header-cell", "类型" }
                                        th { class: "az-table__header-cell", "关联" }
                                        th { class: "az-table__header-cell", "必填" }
                                        th { class: "az-table__header-cell", "唯一" }
                                        th { class: "az-table__header-cell", "操作" }
                                    }
                                }
                                tbody { class: "az-table__body",
                                    if field_views.is_empty() {
                                        tr {
                                            td { class: "az-table__cell az-table__cell--empty", colspan: "8",
                                                "暂无字段 — 点击「添加字段」创建"
                                            }
                                        }
                                    } else {
                                        for (idx, fv) in field_views.iter().enumerate() {
                                            tr {
                                                td { class: "az-table__cell", "{idx + 1}" }
                                                td { class: "az-table__cell", code { "{fv.name}" } }
                                                td { class: "az-table__cell", "{fv.label}" }
                                                td { class: "az-table__cell",
                                                    span { class: "az-badge", "{ft_label(&fv.field_type)}" }
                                                }
                                                td { class: "az-table__cell",
                                                    if fv.field_type == "Relation" {
                                                        span { class: "az-badge az-badge--accent",
                                                            "{rel_label(fv.relation_type.as_deref())}"
                                                        }
                                                        if let Some(ref rmn) = fv.relation_model_name {
                                                            span { " → {rmn}" }
                                                        }
                                                    } else {
                                                        "—"
                                                    }
                                                }
                                                td { class: "az-table__cell",
                                                    if fv.is_required { span { class: "az-badge az-badge--warn", "必填" } }
                                                }
                                                td { class: "az-table__cell",
                                                    if fv.is_unique { span { class: "az-badge", "唯一" } }
                                                }
                                                td { class: "az-table__cell",
                                                    div { style: "display: flex; gap: 4px; align-items: center;",
                                                    details { class: "lowcode-accordion", style: "margin: 0;",
                                                        summary { class: "lowcode-accordion__summary", style: "font-size: 10px; padding: 2px 6px;", "编辑" }
                                                        div { class: "lowcode-accordion__body",
                                                            form {
                                                                method: "get",
                                                                action: "/",
                                                                input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                                                                input { r#type: "hidden", name: "action", value: "edit-field" }
                                                                input { r#type: "hidden", name: "field_id", value: "{fv.id}" }
                                                                input { r#type: "hidden", name: "model", value: "{m.id}" }
                                                                div { class: "settings-form-row",
                                                                    label { "标签" }
                                                                    input { class: "settings-input", name: "field_label", value: "{fv.label}", style: "font-size: 12px;" }
                                                                }
                                                                div { class: "settings-form-row",
                                                                    label { "类型" }
                                                                    select { class: "settings-input", name: "field_type",
                                                                        option { value: "String", selected: fv.field_type == "String", "字符串" }
                                                                        option { value: "Integer", selected: fv.field_type == "Integer", "整数" }
                                                                        option { value: "Float", selected: fv.field_type == "Float", "浮点数" }
                                                                        option { value: "Boolean", selected: fv.field_type == "Boolean", "布尔" }
                                                                        option { value: "DateTime", selected: fv.field_type == "DateTime", "日期时间" }
                                                                        option { value: "Json", selected: fv.field_type == "Json", "JSON" }
                                                                        option { value: "Relation", selected: fv.field_type == "Relation", "关联" }
                                                                    }
                                                                }
                                                                div { class: "settings-form-row",
                                                                    label { "关联类型" }
                                                                    select { class: "settings-input", name: "rel_type",
                                                                        option { value: "", "—" }
                                                                        option { value: "OneToOne", selected: fv.relation_type.as_deref() == Some("OneToOne"), "一对一" }
                                                                        option { value: "OneToMany", selected: fv.relation_type.as_deref() == Some("OneToMany"), "一对多" }
                                                                        option { value: "ManyToMany", selected: fv.relation_type.as_deref() == Some("ManyToMany"), "多对多" }
                                                                        option { value: "SelfRecursive", selected: fv.relation_type.as_deref() == Some("SelfRecursive"), "自递归 · 树" }
                                                                    }
                                                                }
                                                                div { class: "settings-form-row",
                                                                    label { "关联模型" }
                                                                    select { class: "settings-input", name: "rel_model_id",
                                                                        option { value: "", "—" }
                                                                        for rm in &all_models {
                                                                            option { value: "{rm.id}", selected: fv.relation_model_id.as_deref() == Some(rm.id.as_str()), "{rm.label} ({rm.name})" }
                                                                        }
                                                                    }
                                                                }
                                                                div { class: "settings-form-row",
                                                                    label { "默认值" }
                                                                    input { class: "settings-input", name: "def_val", value: "{fv.default_value.as_deref().unwrap_or_default()}", style: "font-size: 11px;", placeholder: "可选" }
                                                                }
                                                                div { class: "settings-form-row", style: "display: flex; gap: 16px; align-items: center;",
                                                                    label { style: "display: flex; align-items: center; gap: 4px;",
                                                                        input { r#type: "checkbox", name: "is_req", value: "1", checked: fv.is_required, style: "width: auto; height: auto;" }
                                                                        "必填"
                                                                    }
                                                                    label { style: "display: flex; align-items: center; gap: 4px;",
                                                                        input { r#type: "checkbox", name: "is_uniq", value: "1", checked: fv.is_unique, style: "width: auto; height: auto;" }
                                                                        "唯一"
                                                                    }
                                                                }
                                                                button { class: "toolbar-button toolbar-button--primary", r#type: "submit", style: "font-size: 11px;", "保存" }
                                                            }
                                                        }
                                                    }
                                                    a {
                                                        href: "/?route={lowcode_route}&model={m.id}&action=delete-field&field_id={fv.id}",
                                                        class: "toolbar-button toolbar-button--danger",
                                                        style: "font-size: 11px; padding: 2px 7px;",
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
                    } else {
                        div { class: "lowcode-detail__header",
                            h2 { "字段定义" }
                            p { class: "lowcode-detail__subtitle", "选择左侧模型查看和管理字段" }
                        }
                        div { class: "lowcode-table-scroll",
                            table { class: "az-table az-table--bordered az-table--dense",
                                thead {
                                    tr {
                                        th { class: "az-table__header-cell", "#" }
                                        th { class: "az-table__header-cell", "字段名" }
                                        th { class: "az-table__header-cell", "标签" }
                                        th { class: "az-table__header-cell", "类型" }
                                        th { class: "az-table__header-cell", "关联" }
                                        th { class: "az-table__header-cell", "必填" }
                                        th { class: "az-table__header-cell", "唯一" }
                                        th { class: "az-table__header-cell", "操作" }
                                    }
                                }
                                tbody { class: "az-table__body",
                                    tr {
                                        td { class: "az-table__cell az-table__cell--empty", colspan: "8",
                                            "← 选择左侧模型以查看字段 · 支持字符串、整数、关联、自递归树等类型"
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

fn parse_q<'a>(query: &'a str, key: &str) -> Option<String> {
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
