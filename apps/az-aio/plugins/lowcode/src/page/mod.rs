pub mod helpers;
pub mod model_editor;
pub mod renderers;
pub mod screen_list;

use az_aio_platform::plugin_api::NativeRenderContext;
use chrono::Utc;
use dioxus::prelude::*;
use uuid::Uuid;

use crate::model::{AppScreen, FormConfig, FormField, MasterDetailConfig, MetaField, MetaModel, TableColumn, TableConfig};
use crate::record::RecordStore;
use crate::store::LowcodeStore;

use helpers::get_store;
use model_editor::render_model_editor;
use renderers::accordion::render_accordion;
use renderers::form::render_form;
use renderers::master_detail::render_master_detail;
use renderers::table::render_table_screen;
use renderers::tree_table::render_tree_table;
use screen_list::render_screen_list_page;

#[allow(non_snake_case)]
pub fn LowcodePage(context: NativeRenderContext) -> Element {
    let route = &context.active_route;

    let model_id = parse_query_param(route, "model");
    let mode = parse_query_param(route, "mode");
    let screen_id = parse_query_param(route, "screen");

    let action = parse_query_param(route, "action");
    let store = get_store();
    handle_action(&store, &action, route);

    if mode.as_deref() == Some("screens") {
        return render_screen_list_page();
    }

    if let Some(ref sid) = screen_id {
        return render_dynamic_screen(sid, route);
    }

    render_model_editor(model_id)
}

fn parse_query_param(route: &str, key: &str) -> Option<String> {
    let query = route.split('?').nth(1)?;
    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        if parts.next()? == key {
            return parts
                .next()
                .map(|v| urlencoding::decode(v).unwrap_or_else(|_| v.into()).into());
        }
    }
    None
}

fn handle_action(store: &LowcodeStore, action: &Option<String>, route: &str) {
    let Some(action) = action.as_deref() else {
        return;
    };
    match action {
        "new-model" => {
            let name = parse_query_param(route, "name").unwrap_or_default();
            let label = parse_query_param(route, "label").unwrap_or_else(|| name.clone());
            let desc = parse_query_param(route, "desc").unwrap_or_default();
            if !name.is_empty() {
                let now = Utc::now().to_rfc3339();
                let model = MetaModel {
                    id: Uuid::new_v4().to_string(),
                    name,
                    label,
                    description: desc,
                    created_at: now.clone(),
                    updated_at: now,
                };
                store.create_model_sync(model);
            }
        }
        "new-field" => {
            let model_id = parse_query_param(route, "model").unwrap_or_default();
            let name = parse_query_param(route, "field_name").unwrap_or_default();
            let label =
                parse_query_param(route, "field_label").unwrap_or_else(|| name.clone());
            let ft = parse_query_param(route, "field_type").unwrap_or_else(|| "String".into());
            let rel_type = parse_query_param(route, "rel_type")
                .filter(|v| !v.is_empty());
            let rel_model_id = parse_query_param(route, "rel_model_id")
                .filter(|v| !v.is_empty());
            if !name.is_empty() && !model_id.is_empty() {
                let now = Utc::now().to_rfc3339();
                let count = store.list_fields_sync(&model_id).len() as i32;
                let field = MetaField {
                    id: Uuid::new_v4().to_string(),
                    model_id,
                    name,
                    label,
                    field_type: ft,
                    relation_type: rel_type,
                    relation_model_id: rel_model_id,
                    is_required: false,
                    is_unique: false,
                    order: count + 1,
                    default_value: None,
                    created_at: now.clone(),
                    updated_at: now,
                };
                store.create_field_sync(&field);
            }
        }
        "delete-field" => {
            let fid = parse_query_param(route, "field_id").unwrap_or_default();
            if !fid.is_empty() {
                store.delete_field_sync(&fid);
            }
        }
        "new-record" => {
            let model_id = parse_query_param(route, "rec_model").unwrap_or_default();
            if !model_id.is_empty() {
                let rec_store = RecordStore::global();
                let mut fields = std::collections::HashMap::new();
                let query = route.split('?').nth(1).unwrap_or("");
                for pair in query.split('&') {
                    let mut parts = pair.splitn(2, '=');
                    let key = parts.next().unwrap_or("");
                    if key.starts_with("rec_")
                        && key != "rec_model"
                        && key != "action"
                        && key != "route"
                        && key != "screen"
                    {
                        let field_key = key.strip_prefix("rec_").unwrap_or(key);
                        let val = parts.next().unwrap_or("");
                        let decoded =
                            urlencoding::decode(val).unwrap_or_else(|_| val.into());
                        fields.insert(field_key.to_string(), decoded.into_owned());
                    }
                }
                if !fields.is_empty() {
                    let _ = rec_store.create(&model_id, fields);
                }
            }
        }
        "delete-record" => {
            let model_id = parse_query_param(route, "rec_model").unwrap_or_default();
            let rec_id = parse_query_param(route, "rec_id").unwrap_or_default();
            if !model_id.is_empty() && !rec_id.is_empty() {
                RecordStore::global().delete(&model_id, &rec_id);
            }
        }
        "edit-record" => {
            let model_id = parse_query_param(route, "rec_model").unwrap_or_default();
            let rec_id = parse_query_param(route, "rec_id").unwrap_or_default();
            if !model_id.is_empty() && !rec_id.is_empty() {
                let rec_store = RecordStore::global();
                let mut fields = std::collections::HashMap::new();
                let query = route.split('?').nth(1).unwrap_or("");
                for pair in query.split('&') {
                    let mut parts = pair.splitn(2, '=');
                    let key = parts.next().unwrap_or("");
                    if key.starts_with("rec_")
                        && key != "rec_model"
                        && key != "rec_id"
                        && key != "action"
                        && key != "route"
                        && key != "screen"
                    {
                        let field_key = key.strip_prefix("rec_").unwrap_or(key);
                        let val = parts.next().unwrap_or("");
                        let decoded =
                            urlencoding::decode(val).unwrap_or_else(|_| val.into());
                        fields.insert(field_key.to_string(), decoded.into_owned());
                    }
                }
                if !fields.is_empty() {
                    rec_store.update(&model_id, &rec_id, fields);
                }
            }
        }
        "delete-model" => {
            let mid = parse_query_param(route, "model").unwrap_or_default();
            if !mid.is_empty() {
                store.delete_model_sync(&mid);
            }
        }
        "edit-field" => {
            let fid = parse_query_param(route, "field_id").unwrap_or_default();
            let label = parse_query_param(route, "field_label").unwrap_or_default();
            let ft = parse_query_param(route, "field_type").unwrap_or_default();
            let rel_type = parse_query_param(route, "rel_type");
            let rel_model_id = parse_query_param(route, "rel_model_id");
            if !fid.is_empty() {
                let mut fields = store.mem_fields_sync();
                if let Some(f) = fields.iter_mut().find(|f| f.id == fid) {
                    if !label.is_empty() {
                        f.label = label;
                    }
                    if !ft.is_empty() {
                        f.field_type = ft;
                    }
                    f.relation_type = rel_type.filter(|v| !v.is_empty());
                    f.relation_model_id = rel_model_id.filter(|v| !v.is_empty());
                    f.updated_at = Utc::now().to_rfc3339();
                    store.update_field_sync_v(f);
                }
            }
        }
        "delete-screen" => {
            let sid = parse_query_param(route, "scr_id").unwrap_or_default();
            if !sid.is_empty() {
                store.delete_screen_sync(&sid);
            }
        }
        "edit-screen" => {
            let sid = parse_query_param(route, "scr_id").unwrap_or_default();
            let label = parse_query_param(route, "scr_label").unwrap_or_default();
            if !sid.is_empty() && !label.is_empty() {
                store.update_screen_label_sync(&sid, &label);
            }
        }
        "new-screen" => {
            let name = parse_query_param(route, "scr_name").unwrap_or_default();
            let label =
                parse_query_param(route, "scr_label").unwrap_or_else(|| name.clone());
            let layout =
                parse_query_param(route, "scr_layout").unwrap_or_else(|| "Table".into());
            let model_id = parse_query_param(route, "scr_model_id").unwrap_or_default();
            if !name.is_empty() && !model_id.is_empty() {
                let fields = store.list_fields_sync(&model_id);
                let config_json = auto_config_json(&layout, &fields);
                let now = Utc::now().to_rfc3339();
                let screen = AppScreen {
                    id: Uuid::new_v4().to_string(),
                    name,
                    label,
                    layout,
                    model_id,
                    config_json,
                    created_at: now.clone(),
                    updated_at: now,
                };
                store.create_screen_sync(screen);
            }
        }
        _ => {}
    }
}

fn render_dynamic_screen(screen_id: &str, route: &str) -> Element {
    let store = get_store();
    let screen_opt = store.get_screen_sync(screen_id);
    let lowcode_route = "/lowcode";
    let back_href = format!("/?route={lowcode_route}&mode=screens");

    match screen_opt {
        Some(screen) => {
            let fields = store.list_fields_sync(&screen.model_id);
            let query = route.split('?').nth(1).unwrap_or("");
            match screen.layout.as_str() {
                "Table" => {
                    render_table_screen(&screen.label, &screen.model_id, &fields, &screen.config_json, query)
                }
                "MasterDetail" => render_master_detail(
                    &screen.label,
                    &screen.model_id,
                    &fields,
                    &screen.config_json,
                    query,
                ),
                "Accordion" => {
                    render_accordion(&screen.label, &screen.model_id, &fields)
                }
                "Form" => {
                    render_form(&screen.label, &screen.model_id, &fields, &screen.config_json)
                }
                "TreeTable" => {
                    render_tree_table(&screen.label, &screen.model_id, &fields)
                }
                _ => rsx! {
                    section { class: "lowcode-page",
                        header { class: "lowcode-page__header",
                            h1 { "{screen.label}" }
                            p { "未支持的布局: {screen.layout}" }
                        }
                        a { href: "{back_href}", class: "toolbar-button", "← 返回" }
                    }
                },
            }
        }
        None => rsx! {
            section { class: "lowcode-page",
                header { class: "lowcode-page__header", h1 { "未找到 AppScreen" } }
                a { href: "{back_href}", class: "toolbar-button", "← 返回" }
            }
        },
    }
}

/// Auto-generate config_json from model fields for the given layout.
fn auto_config_json(layout: &str, fields: &[crate::model::MetaFieldView]) -> String {
    let non_rel: Vec<&crate::model::MetaFieldView> =
        fields.iter().filter(|f| f.field_type != "Relation").collect();

    match layout {
        "Table" => serde_json::to_string(&TableConfig {
            columns: non_rel
                .iter()
                .map(|f| TableColumn {
                    field_name: f.name.clone(),
                    label: f.label.clone(),
                    sortable: false,
                    width: None,
                })
                .collect(),
            searchable_fields: vec![],
            page_size: 20,
        })
        .unwrap_or_default(),
        "MasterDetail" => {
            let tree_field = fields
                .iter()
                .find(|f| f.relation_type.as_deref() == Some("SelfRecursive"))
                .map(|f| f.id.clone())
                .unwrap_or_default();
            serde_json::to_string(&MasterDetailConfig {
                tree_field_id: tree_field,
                detail_columns: non_rel
                    .iter()
                    .map(|f| TableColumn {
                        field_name: f.name.clone(),
                        label: f.label.clone(),
                        sortable: false,
                        width: None,
                    })
                    .collect(),
                detail_searchable: vec![],
            })
            .unwrap_or_default()
        }
        "Form" => serde_json::to_string(&FormConfig {
            fields: non_rel
                .iter()
                .map(|f| FormField {
                    field_name: f.name.clone(),
                    label: f.label.clone(),
                    field_type: f.field_type.clone(),
                    required: f.is_required,
                    placeholder: format!("输入{}", f.label),
                    options: vec![],
                })
                .collect(),
            submit_label: "保存".into(),
        })
        .unwrap_or_default(),
        "Accordion" | "TreeTable" => "{}".into(),
        _ => "{}".into(),
    }
}
