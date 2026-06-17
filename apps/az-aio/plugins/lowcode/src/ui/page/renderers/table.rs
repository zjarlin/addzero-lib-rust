use az_dioxus_components::{
    accordion::Accordion,
    toolbar_button::{ToolbarButton, ToolbarButtonLink, ToolbarButtonTone},
    form::{ActionForm, FormRow, HiddenInput, Input},
    table::{Table, TableBody, TableCell, TableHead, TableHeaderCell, TableRow},
    workbench::{PageHeader, TableViewport, WorkbenchPage},
};
use dioxus::prelude::*;

use crate::backend::model::{MetaFieldView, TableColumn, TableConfig};
use crate::backend::record::{RecordStore, RecordWithId};
use crate::ui::page::helpers::{
    parse_query, render_enum_select, render_enum_select_edit, render_rel_select,
    render_rel_select_edit, resolve_cell, LowcodeActionForm,
};

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
        frozen_header: true,
        frozen_columns: 1,
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
        WorkbenchPage {
            PageHeader {
                title: title.to_string(),
                subtitle: format!("{total} 条记录 · 第 {page}/{} 页", total_pages.max(1)),
                ToolbarButtonLink { href: format!("/?route={lowcode_route}&mode=screens"), "← 返回" }

                // New record form
                Accordion { title: "＋ 新建记录",
                        LowcodeActionForm {
                            action_name: "new-record",
                            hidden_fields: vec![
                                ("rec_model".to_string(), model_id.to_string()),
                                ("screen".to_string(), screen_id.clone()),
                            ],
                            for f in fields.iter() {
                                FormRow { label: f.label.clone(),
                                    if f.field_type == "Enum" {
                                        {render_enum_select(f)}
                                    } else if f.field_type == "Relation" {
                                        {render_rel_select(f, &rec_store)}
                                    } else {
                                        Input { name: format!("rec_{}", f.name), placeholder: format!("输入{}", f.label) }
                                    }
                                }
                            }
                            ToolbarButton { tone: ToolbarButtonTone::Primary, button_type: "submit", "创建" }
                        }
                }

                // Search
                ActionForm {
                    div { style: "display:flex; gap:8px; align-items:center; padding:8px 0;",
                        HiddenInput { name: "route", value: lowcode_route }
                        HiddenInput { name: "screen", value: screen_id.clone() }
                        Input { name: "search", placeholder: "搜索...", value: search.clone(), style: "max-width:280px;" }
                        ToolbarButton { button_type: "submit", "搜索" }
                        if !search.is_empty() {
                            ToolbarButtonLink { href: action_base.clone(), "清除" }
                        }
                    }
                }
            }

            // Batch delete form
            LowcodeActionForm {
                id: "batch-form",
                action_name: "batch-delete-record",
                hidden_fields: vec![
                    ("rec_model".to_string(), model_id.to_string()),
                    ("screen".to_string(), screen_id.clone()),
                ],
                HiddenInput { name: "ids", id: "batch-ids" }

                TableViewport {
                    Table {
                        class: table_class(&config),
                        frozen_header: false,
                        TableHead {
                            TableRow {
                                TableHeaderCell { class: frozen_header_class_extra(&config, 0), style: frozen_style(&config, 0, "32px", "width:32px; text-align:center;"),
                                    input { r#type: "checkbox", id: "select-all", "onchange": "document.querySelectorAll('.row-checkbox').forEach(cb=>cb.checked=this.checked);" }
                                }
                                for (index, col) in config.columns.iter().enumerate() {
                                    {
                                        let column_index = index + 1;
                                        let width = col.width.as_deref().unwrap_or("160px");
                                        rsx! {
                                    TableHeaderCell { class: frozen_header_class_extra(&config, column_index), style: frozen_style(&config, column_index, width, ""),
                                        a {
                                            href: "{make_sort_link(&col.field_name)}",
                                            style: "color:inherit; text-decoration:none;",
                                            "{col.label}{sort_indicator(&col.field_name)}"
                                        }
                                    }
                                        }
                                    }
                                }
                                TableHeaderCell { style: "width:110px; text-align:center;", "操作" }
                            }
                        }
                        TableBody {
                            if paged.is_empty() {
                                TableRow {
                                    TableCell { class: "table-view__cell--empty", colspan: col_len + 1,
                                        if search.is_empty() { "暂无记录" } else { "无匹配记录" }
                                    }
                                }
                            } else {
                                for rec in &paged {
                                    TableRow {
                                        TableCell { class: frozen_cell_class_extra(&config, 0), style: frozen_style(&config, 0, "32px", "text-align:center;"),
                                            input { class: "row-checkbox", r#type: "checkbox", value: "{rec.id}", "onchange": "updateBatchIds()" }
                                        }
                                        for (index, cn) in col_names.iter().enumerate() {
                                            {
                                                let column_index = index + 1;
                                                let width = config.columns.get(index).and_then(|column| column.width.as_deref()).unwrap_or("160px");
                                                rsx! {
                                            TableCell { class: frozen_cell_class_extra(&config, column_index), style: frozen_style(&config, column_index, width, ""),
                                                "{resolve_cell(fields, *cn, rec.fields.get(*cn).cloned().unwrap_or_default())}"
                                            }
                                                }
                                            }
                                        }
                                        TableCell { style: "text-align:center; white-space:nowrap;",
                                            Accordion { title: "编辑", class: "accordion--inline", summary_class: "accordion__summary--compact",
                                                    LowcodeActionForm {
                                                        action_name: "edit-record",
                                                        hidden_fields: vec![
                                                            ("rec_model".to_string(), model_id.to_string()),
                                                            ("rec_id".to_string(), rec.id.clone()),
                                                            ("screen".to_string(), screen_id.clone()),
                                                        ],
                                                        for fv in fields.iter() {
                                                            FormRow { label: fv.label.clone(),
                                                                if fv.field_type == "Enum" {
                                                                    {render_enum_select_edit(fv, rec.fields.get(fv.name.as_str()).cloned().unwrap_or_default())}
                                                                } else if fv.field_type == "Relation" {
                                                                    {render_rel_select_edit(fv, &rec_store, rec.fields.get(fv.name.as_str()).cloned().unwrap_or_default())}
                                                                } else {
                                                                    Input {
                                                                        name: format!("rec_{}", fv.name),
                                                                        value: rec.fields.get(fv.name.as_str()).cloned().unwrap_or_default(),
                                                                        class: "form-input--compact",
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        ToolbarButton { tone: ToolbarButtonTone::Primary, button_type: "submit", "保存" }
                                                    }
                                            }
                                            ToolbarButtonLink {
                                                href: format!("{action_base}&action=delete-record&rec_model={model_id}&rec_id={}", rec.id),
                                                tone: ToolbarButtonTone::Danger,
                                                class: "toolbar-button--table-gap",
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

fn table_class(config: &TableConfig) -> &'static str {
    if config.frozen_header {
        "table-view--bordered table-view--dense table-view--frozen-header"
    } else {
        "table-view--bordered table-view--dense"
    }
}

fn frozen_header_class_extra(config: &TableConfig, index: usize) -> &'static str {
    if column_is_frozen(config, index) {
        "table-view__cell--frozen"
    } else {
        ""
    }
}

fn frozen_cell_class_extra(config: &TableConfig, index: usize) -> &'static str {
    if column_is_frozen(config, index) {
        "table-view__cell--frozen"
    } else {
        ""
    }
}

fn frozen_style(config: &TableConfig, index: usize, width: &str, base: &str) -> String {
    if !column_is_frozen(config, index) {
        return base.to_string();
    }
    let left = if index == 0 {
        "0px".to_string()
    } else {
        format!("calc(32px + {} * 160px)", index - 1)
    };
    if base.is_empty() {
        format!("left:{left}; min-width:{width};")
    } else {
        format!("{base} left:{left}; min-width:{width};")
    }
}

fn column_is_frozen(config: &TableConfig, index: usize) -> bool {
    if config.frozen_columns == 0 {
        return false;
    }
    index == 0 || index <= config.frozen_columns
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
                ToolbarButtonLink { href: make_link(page - 1), class: "toolbar-button--compact", "‹ 上一页" }
            } else {
                ToolbarButton { disabled: true, class: "toolbar-button--compact toolbar-button--disabled", "‹ 上一页" }
            }
            for p in &pages {
                if *p == page {
                    ToolbarButton { tone: ToolbarButtonTone::Primary, disabled: true, class: "toolbar-button--compact toolbar-button--page", "{p}" }
                } else {
                    ToolbarButtonLink { href: make_link(*p), class: "toolbar-button--compact toolbar-button--page", "{p}" }
                }
            }
            if page < total {
                ToolbarButtonLink { href: make_link(page + 1), class: "toolbar-button--compact", "下一页 ›" }
            } else {
                ToolbarButton { disabled: true, class: "toolbar-button--compact toolbar-button--disabled", "下一页 ›" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frozen_columns_apply_to_first_configured_columns() {
        let config = TableConfig {
            columns: Vec::new(),
            searchable_fields: Vec::new(),
            page_size: 20,
            frozen_header: true,
            frozen_columns: 2,
        };

        assert_eq!(frozen_header_class_extra(&config, 1), "table-view__cell--frozen");
        assert_eq!(frozen_cell_class_extra(&config, 2), "table-view__cell--frozen");
        assert_eq!(frozen_cell_class_extra(&config, 3), "");
    }
}
