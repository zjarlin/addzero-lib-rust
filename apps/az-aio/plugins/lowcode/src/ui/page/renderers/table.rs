use dioxus::prelude::*;

use crate::backend::model::{MetaFieldView, TableColumn, TableConfig};
use crate::ui::page::helpers::{
    parse_query, render_enum_select, render_enum_select_edit, render_rel_select,
    render_rel_select_edit, resolve_cell,
};
use crate::backend::record::{RecordStore, RecordWithId};

pub fn render_table_screen(
    title: &str,
    model_id: &str,
    fields: &[MetaFieldView],
    config_json: &str,
    query: &str,
) -> Element {
    let config: TableConfig = serde_json::from_str(config_json).unwrap_or_else(|_| TableConfig {
        columns: fields
            .iter()
            .filter(|f| f.field_type != "Relation")
            .map(|f| TableColumn {
                field_name: f.name.clone(),
                label: f.label.clone(),
                sortable: false,
                width: None,
            })
            .collect(),
        searchable_fields: vec![],
        page_size: 20,
    });

    let rec_store = RecordStore::global();
    let all_records = rec_store.list(model_id);
    let lowcode_route = "/lowcode";
    let screen_id = parse_query(query, "screen").unwrap_or_default();
    let search = parse_query(query, "search").unwrap_or_default();
    let page: usize = parse_query(query, "page")
        .and_then(|p| p.parse().ok())
        .unwrap_or(1);
    let sort_field = parse_query(query, "sort").unwrap_or_default();
    let sort_asc = parse_query(query, "order").as_deref() != Some("desc");

    // Filter
    let mut records: Vec<&RecordWithId> = if search.is_empty() {
        all_records.iter().collect()
    } else {
        let q = search.to_lowercase();
        all_records
            .iter()
            .filter(|r| r.fields.values().any(|v| v.to_lowercase().contains(&q)))
            .collect()
    };

    // Sort
    if !sort_field.is_empty() {
        records.sort_by(|a, b| {
            let va = a.fields.get(&sort_field).map(|s| s.as_str()).unwrap_or("");
            let vb = b.fields.get(&sort_field).map(|s| s.as_str()).unwrap_or("");
            if let (Ok(na), Ok(nb)) = (va.parse::<f64>(), vb.parse::<f64>()) {
                if sort_asc {
                    na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    nb.partial_cmp(&na).unwrap_or(std::cmp::Ordering::Equal)
                }
            } else {
                if sort_asc { va.cmp(vb) } else { vb.cmp(va) }
            }
        });
    }

    let total = records.len();
    let page_size = config.page_size.max(1);
    let total_pages = (total + page_size - 1) / page_size;
    let page = page.min(total_pages.max(1));
    let start = (page - 1) * page_size;
    let paged: Vec<&&RecordWithId> = records.iter().skip(start).take(page_size).collect();

    let col_names: Vec<&str> = config
        .columns
        .iter()
        .map(|c| c.field_name.as_str())
        .collect();
    let col_len = col_names.len() + 1;

    let make_sort_link = |field: &str| -> String {
        let next_asc = if sort_field == field && sort_asc {
            "desc"
        } else {
            "asc"
        };
        let mut params = vec![
            ("route", lowcode_route),
            ("screen", &screen_id),
            ("sort", field),
            ("order", next_asc),
        ];
        if !search.is_empty() {
            params.push(("search", &search));
        }
        let qs: Vec<String> = params.iter().map(|(k, v)| format!("{k}={v}")).collect();
        format!("/?{}", qs.join("&"))
    };

    let make_page_link = |p: usize| -> String {
        let mut params = vec![("route", lowcode_route), ("screen", &screen_id)];
        if !search.is_empty() {
            params.push(("search", &search));
        }
        if !sort_field.is_empty() {
            params.push(("sort", &sort_field));
            params.push(("order", if sort_asc { "asc" } else { "desc" }));
        }
        let page_str = p.to_string();
        if p > 1 {
            params.push(("page", &page_str));
        }
        let qs: Vec<String> = params
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        format!("/?{}", qs.join("&"))
    };

    let action_base = format!("/?route={lowcode_route}&screen={screen_id}");
    let batch_onclick = "var ids=[];document.querySelectorAll('.row-checkbox:checked').forEach(cb=>ids.push(cb.value));if(ids.length===0){alert('请先选择记录');return;}if(!confirm('确定删除选中的 '+ids.length+' 条记录?'))return;document.getElementById('batch-ids').value=ids.join(',');document.getElementById('batch-form').submit();";
    let sort_indicator = |field: &str| -> &str {
        if sort_field == field {
            if sort_asc { " ▲" } else { " ▼" }
        } else {
            ""
        }
    };

    rsx! {
        section { class: "lowcode-page",
            header { class: "lowcode-page__header",
                h1 { "{title}" }
                p { "{total} 条记录 · 第 {page}/{total_pages.max(1)} 页" }
                a { href: "/?route={lowcode_route}&mode=screens", class: "toolbar-button", "← 返回" }

                // New record form
                details { class: "lowcode-accordion",
                    summary { class: "lowcode-accordion__summary", "＋ 新建记录" }
                    div { class: "lowcode-accordion__body",
                        form { method: "get", action: "/",
                            input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                            input { r#type: "hidden", name: "action", value: "new-record" }
                            input { r#type: "hidden", name: "rec_model", value: "{model_id}" }
                            input { r#type: "hidden", name: "screen", value: "{screen_id}" }
                            for f in fields.iter() {
                                div { class: "settings-form-row",
                                    label { "{f.label}" }
                                    if f.field_type == "Enum" {
                                        {render_enum_select(f)}
                                    } else if f.field_type == "Relation" {
                                        {render_rel_select(f, &rec_store)}
                                    } else {
                                        input { class: "settings-input", name: "rec_{f.name}", placeholder: "输入{f.label}" }
                                    }
                                }
                            }
                            button { class: "toolbar-button toolbar-button--primary", r#type: "submit", "创建" }
                        }
                    }
                }

                // Search
                form { method: "get", action: "/",
                    div { style: "display:flex; gap:8px; align-items:center; padding:8px 0;",
                        input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                        input { r#type: "hidden", name: "screen", value: "{screen_id}" }
                        input { class: "settings-input", name: "search", placeholder: "搜索...", value: "{search}", style: "max-width:280px;" }
                        button { class: "toolbar-button", r#type: "submit", "搜索" }
                        if !search.is_empty() {
                            a { href: "{action_base}", class: "toolbar-button", style: "font-size:11px;", "清除" }
                        }
                    }
                }
            }

            // Batch delete form
            form { method: "get", action: "/", id: "batch-form",
                input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                input { r#type: "hidden", name: "action", value: "batch-delete-record" }
                input { r#type: "hidden", name: "rec_model", value: "{model_id}" }
                input { r#type: "hidden", name: "screen", value: "{screen_id}" }
                input { r#type: "hidden", name: "ids", id: "batch-ids" }

                div { class: "lowcode-table-scroll",
                    table { class: "az-table az-table--bordered az-table--dense",
                        thead {
                            tr {
                                th { class: "az-table__header-cell", style: "width:32px; text-align:center;",
                                    input { r#type: "checkbox", id: "select-all", "onchange": "document.querySelectorAll('.row-checkbox').forEach(cb=>cb.checked=this.checked);" }
                                }
                                for col in &config.columns {
                                    th { class: "az-table__header-cell",
                                        a {
                                            href: "{make_sort_link(&col.field_name)}",
                                            style: "color:inherit; text-decoration:none;",
                                            "{col.label}{sort_indicator(&col.field_name)}"
                                        }
                                    }
                                }
                                th { class: "az-table__header-cell", style: "width:110px; text-align:center;", "操作" }
                            }
                        }
                        tbody { class: "az-table__body",
                            if paged.is_empty() {
                                tr {
                                    td { class: "az-table__cell az-table__cell--empty", colspan: "{col_len + 1}",
                                        if search.is_empty() { "暂无记录" } else { "无匹配记录" }
                                    }
                                }
                            } else {
                                for rec in &paged {
                                    tr {
                                        td { class: "az-table__cell", style: "text-align:center;",
                                            input { class: "row-checkbox", r#type: "checkbox", value: "{rec.id}", "onchange": "updateBatchIds()" }
                                        }
                                        for cn in &col_names {
                                            td { class: "az-table__cell",
                                                "{resolve_cell(fields, *cn, rec.fields.get(*cn).cloned().unwrap_or_default())}"
                                            }
                                        }
                                        td { class: "az-table__cell", style: "text-align:center; white-space:nowrap;",
                                            details { class: "lowcode-accordion", style: "margin:0; display:inline-block;",
                                                summary { class: "lowcode-accordion__summary", style: "font-size:11px; padding:2px 6px;", "编辑" }
                                                div { class: "lowcode-accordion__body",
                                                    form { method: "get", action: "/",
                                                        input { r#type: "hidden", name: "route", value: "{lowcode_route}" }
                                                        input { r#type: "hidden", name: "action", value: "edit-record" }
                                                        input { r#type: "hidden", name: "rec_model", value: "{model_id}" }
                                                        input { r#type: "hidden", name: "rec_id", value: "{rec.id}" }
                                                        input { r#type: "hidden", name: "screen", value: "{screen_id}" }
                                                        for fv in fields.iter() {
                                                            div { class: "settings-form-row",
                                                                label { "{fv.label}" }
                                                                if fv.field_type == "Enum" {
                                                                    {render_enum_select_edit(fv, rec.fields.get(fv.name.as_str()).cloned().unwrap_or_default())}
                                                                } else if fv.field_type == "Relation" {
                                                                    {render_rel_select_edit(fv, &rec_store, rec.fields.get(fv.name.as_str()).cloned().unwrap_or_default())}
                                                                } else {
                                                                    input { class: "settings-input", name: "rec_{fv.name}", value: "{rec.fields.get(fv.name.as_str()).cloned().unwrap_or_default()}", style: "font-size:12px;" }
                                                                }
                                                            }
                                                        }
                                                        button { class: "toolbar-button toolbar-button--primary", r#type: "submit", style: "font-size:11px;", "保存" }
                                                    }
                                                }
                                            }
                                            a {
                                                href: "{action_base}&action=delete-record&rec_model={model_id}&rec_id={rec.id}",
                                                class: "toolbar-button toolbar-button--danger",
                                                style: "font-size:11px; padding:2px 7px; margin-left:4px;",
                                                "删除"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Batch delete button + pagination
                div { style: "display:flex; align-items:center; justify-content:space-between; padding:8px 0;",
                    button {
                        class: "toolbar-button toolbar-button--danger",
                        r#type: "button",
                        style: "font-size:11px;",
                        "onclick": batch_onclick ,
                        "批量删除选中"
                    }
                    {render_pagination(page, total_pages, &make_page_link)}
                }
            }
        }
        script { {r#"
function updateBatchIds() {
    var ids = [];
    document.querySelectorAll('.row-checkbox:checked').forEach(function(cb) { ids.push(cb.value); });
    var all = document.querySelectorAll('.row-checkbox');
    document.getElementById('select-all').checked = all.length > 0 && document.querySelectorAll('.row-checkbox:checked').length === all.length;
}
"#} }
    }
}

fn render_pagination(page: usize, total: usize, make_link: &dyn Fn(usize) -> String) -> Element {
    if total <= 1 {
        return rsx! {};
    }

    let pages: Vec<usize> = {
        let start = if page > 3 { page - 3 } else { 1 };
        let end = (start + 6).min(total);
        (start..=end).collect()
    };

    rsx! {
        div { style: "display:flex; gap:4px; align-items:center;",
            if page > 1 {
                a { href: "{make_link(page - 1)}", class: "toolbar-button", style: "font-size:11px;", "‹ 上一页" }
            } else {
                span { class: "toolbar-button", style: "font-size:11px; opacity:0.4;", "‹ 上一页" }
            }
            for p in &pages {
                if *p == page {
                    span { class: "toolbar-button toolbar-button--primary", style: "font-size:11px; min-width:24px; text-align:center;", "{p}" }
                } else {
                    a { href: "{make_link(*p)}", class: "toolbar-button", style: "font-size:11px; min-width:24px; text-align:center;", "{p}" }
                }
            }
            if page < total {
                a { href: "{make_link(page + 1)}", class: "toolbar-button", style: "font-size:11px;", "下一页 ›" }
            } else {
                span { class: "toolbar-button", style: "font-size:11px; opacity:0.4;", "下一页 ›" }
            }
        }
    }
}
