#![allow(non_snake_case)]

//! lowcode 插件的 engine Admin 页面。

use az_aio_platform::plugin::api::NativeRenderContext;
use az_dioxus_components::prelude::*;
use az_engine::{DataRecordView, HookDefinition, MetaField, MetaModel, PageData, PageParams};
use dioxus::prelude::*;
use serde_json::Value;

use crate::state::{run_engine_future, store};

const ACTION_ENDPOINT: &str = "/api/engine/ui-action";

struct PageSnapshot {
    models: Vec<MetaModel>,
    fields: Vec<MetaField>,
    hooks: Vec<HookDefinition>,
    records: Vec<DataRecordView>,
    total_records: u64,
    selected_model: Option<String>,
    tab: String,
    error: Option<String>,
}

/// 渲染 engine 的模型、字段、钩子和记录工作台。
pub fn LowcodePage(context: NativeRenderContext) -> Element {
    let snapshot = load_snapshot(&context.active_route);

    rsx! {
        WorkbenchPage { class: "engine-page",
            PageHeader {
                title: "engine".to_string(),
                subtitle: "PostgreSQL 元模型与动态记录".to_string(),
                div { class: "toolbar",
                    a { class: "toolbar-button toolbar-button--primary", href: "/?route=/lowcode&tab=fields", "模型" }
                    a { class: "toolbar-button", href: tab_href(snapshot.selected_model.as_deref(), "fields"), "字段" }
                    a { class: "toolbar-button", href: tab_href(snapshot.selected_model.as_deref(), "hooks"), "钩子" }
                    a { class: "toolbar-button", href: tab_href(snapshot.selected_model.as_deref(), "records"), "记录" }
                }
            }
            if let Some(error) = &snapshot.error {
                div { class: "settings-alert settings-alert--danger", "{error}" }
            }
            SplitWorkbench {
                WorkbenchTree {
                    WorkbenchTreeHeader { title: "模型树".to_string(),
                        span { class: "status-badge", "{snapshot.models.len()}" }
                    }
                    WorkbenchTreeList {
                        for model in snapshot.models.iter() {
                            a {
                                class: if snapshot.selected_model.as_deref() == Some(model.name.as_str()) { "nav-button nav-button--active" } else { "nav-button" },
                                href: tab_href(Some(&model.name), &snapshot.tab),
                                span { class: "nav-button__icon", "▤" }
                                span { class: "nav-button__label", "{model.display_name}" }
                                span { class: "nav-button__detail", "{model.name}" }
                            }
                        }
                    }
                    {render_model_form()}
                }
                WorkbenchDetail {
                    if let Some(model_name) = snapshot.selected_model.as_deref() {
                        {render_detail_header(&snapshot, model_name)}
                        if snapshot.tab == "hooks" {
                            {render_hooks(model_name, &snapshot.hooks)}
                        } else if snapshot.tab == "records" {
                            {render_records(model_name, &snapshot.fields, &snapshot.records, snapshot.total_records)}
                        } else {
                            {render_fields(model_name, &snapshot.fields)}
                        }
                    } else {
                        WorkbenchDetailHeader {
                            title: "模型".to_string(),
                            subtitle: "创建或选择一个模型".to_string(),
                        }
                    }
                }
            }
        }
    }
}

fn load_snapshot(route: &str) -> PageSnapshot {
    let selected_model = parse_query_param(route, "model");
    let tab = match parse_query_param(route, "tab") {
        Some(value) => value,
        None => "fields".to_string(),
    };
    let route_error = parse_query_param(route, "error");
    let mut error = route_error;
    let mut models = Vec::new();
    let mut fields = Vec::new();
    let mut hooks = Vec::new();
    let mut records = Vec::new();
    let mut total_records = 0;

    match store().and_then(|store| {
        run_engine_future(async move {
            let page = store.list_models(PageParams { o: 0, s: 200 }).await?;
            Ok((store, page.d))
        })
    }) {
        Ok((engine_store, loaded_models)) => {
            models = loaded_models;
            let model_name = selected_model
                .clone()
                .or_else(|| models.first().map(|model| model.name.clone()));
            if let Some(model_name) = model_name.as_deref() {
                let result = run_engine_future(async move {
                    let loaded_fields = engine_store.list_fields(model_name).await?;
                    let loaded_hooks = engine_store.list_hooks(model_name).await?;
                    let loaded_records: PageData<DataRecordView> = engine_store
                        .executor()
                        .list_records(model_name, PageParams { o: 0, s: 50 })
                        .await?;
                    Ok((loaded_fields, loaded_hooks, loaded_records))
                });
                match result {
                    Ok((loaded_fields, loaded_hooks, loaded_records)) => {
                        fields = loaded_fields;
                        hooks = loaded_hooks;
                        total_records = loaded_records.t;
                        records = loaded_records.d;
                    }
                    Err(load_error) => error = Some(load_error.to_string()),
                }
            }
            PageSnapshot {
                models,
                fields,
                hooks,
                records,
                total_records,
                selected_model: model_name,
                tab,
                error,
            }
        }
        Err(load_error) => PageSnapshot {
            models,
            fields,
            hooks,
            records,
            total_records,
            selected_model,
            tab,
            error: Some(load_error.to_string()),
        },
    }
}

fn render_detail_header(snapshot: &PageSnapshot, model_name: &str) -> Element {
    let model = snapshot
        .models
        .iter()
        .find(|model| model.name == model_name);
    let title = match model.map(|model| model.display_name.clone()) {
        Some(value) => value,
        None => model_name.to_string(),
    };
    let subtitle = format!(
        "{} fields · {} hooks · {} records",
        snapshot.fields.len(),
        snapshot.hooks.len(),
        snapshot.total_records
    );
    let display_name = model
        .map(|model| model.display_name.clone())
        .unwrap_or_else(|| model_name.to_string());

    rsx! {
        WorkbenchDetailHeader { title, subtitle,
            ActionForm {
                method: "post",
                action: ACTION_ENDPOINT,
                class: "toolbar",
                HiddenInput { name: "action", value: "update_model" }
                HiddenInput { name: "model_name", value: model_name }
                HiddenInput { name: "name", value: model_name }
                Input { name: "display_name", value: display_name, required: true }
                button { class: "toolbar-button toolbar-button--primary", r#type: "submit", "保存模型" }
            }
            ActionForm {
                method: "post",
                action: ACTION_ENDPOINT,
                class: "toolbar",
                HiddenInput { name: "action", value: "delete_model" }
                HiddenInput { name: "model_name", value: model_name }
                button { class: "toolbar-button toolbar-button--danger", r#type: "submit", "删除模型" }
            }
        }
    }
}

fn render_model_form() -> Element {
    rsx! {
        ActionForm {
            method: "post",
            action: ACTION_ENDPOINT,
            class: "settings-form",
            HiddenInput { name: "action", value: "create_model" }
            FormGrid {
                FormRow { label: "name".to_string(), required: true,
                    Input { name: "name", required: true, placeholder: "order" }
                }
                FormRow { label: "display".to_string(), required: true,
                    Input { name: "display_name", required: true, placeholder: "订单" }
                }
            }
            button { class: "toolbar-button toolbar-button--primary", r#type: "submit", "新建模型" }
        }
    }
}

fn render_fields(model_name: &str, fields: &[MetaField]) -> Element {
    let dependency_placeholder = r#"[{"alias":"vip","source_model_name":"user","local_field":"user_id","source_payload_field":"vip"}]"#;
    rsx! {
        div { class: "settings-section",
            ActionForm {
                method: "post",
                action: ACTION_ENDPOINT,
                class: "settings-form",
                HiddenInput { name: "action", value: "create_field" }
                HiddenInput { name: "model_name", value: model_name }
                FormGrid { wide: true,
                    FormRow { label: "name".to_string(), required: true,
                        Input { name: "name", required: true, placeholder: "amount" }
                    }
                    FormRow { label: "display".to_string(), required: true,
                        Input { name: "display_name", required: true, placeholder: "金额" }
                    }
                    FormRow { label: "type".to_string(), required: true,
                        Select {
                            name: "field_type",
                            required: true,
                            options: field_type_options("string"),
                        }
                    }
                    FormRow { label: "order".to_string(),
                        Input { input_type: "number", name: "order_index", value: "0" }
                    }
                    FormRow { label: "expression".to_string(), wide: true,
                        Input { name: "expression", placeholder: "amount * 2" }
                    }
                    FormRow { label: "dependency".to_string(), wide: true,
                        Input { name: "dependency_json", placeholder: dependency_placeholder }
                    }
                    CheckboxRow { name: "is_required", label: "必填".to_string() }
                }
                button { class: "toolbar-button toolbar-button--primary", r#type: "submit", "添加字段" }
            }
            TableViewport {
                Table { dense: true, striped: true, bordered: true,
                    TableHead {
                        TableRow {
                            TableHeaderCell { "字段" }
                            TableHeaderCell { "类型" }
                            TableHeaderCell { "必填" }
                            TableHeaderCell { "表达式" }
                            TableHeaderCell { "操作" }
                        }
                    }
                    TableBody {
                        for field in fields {
                            {render_field_edit_row(model_name, field)}
                        }
                    }
                }
            }
        }
    }
}

fn render_field_edit_row(model_name: &str, field: &MetaField) -> Element {
    rsx! {
        TableRow {
            TableCell {
                ActionForm { method: "post", action: ACTION_ENDPOINT,
                    class: "settings-form",
                    HiddenInput { name: "action", value: "update_field" }
                    HiddenInput { name: "model_name", value: model_name }
                    HiddenInput { name: "field_id", value: field.id.clone() }
                    FormGrid { wide: true,
                        FormRow { label: "name".to_string(), required: true,
                            Input { name: "name", value: field.name.clone(), required: true }
                        }
                        FormRow { label: "display".to_string(), required: true,
                            Input { name: "display_name", value: field.display_name.clone(), required: true }
                        }
                        FormRow { label: "type".to_string(), required: true,
                            Select {
                                name: "field_type",
                                required: true,
                                options: field_type_options(&field.field_type),
                            }
                        }
                        FormRow { label: "order".to_string(),
                            Input { input_type: "number", name: "order_index", value: field.order_index.to_string() }
                        }
                        FormRow { label: "expression".to_string(), wide: true,
                            Input { name: "expression", value: optional_text(&field.expression) }
                        }
                        FormRow { label: "dependency".to_string(), wide: true,
                            Input { name: "dependency_json", value: optional_text(&field.dependency_json) }
                        }
                        CheckboxRow { name: "is_required", label: "必填".to_string(), checked: field.is_required }
                    }
                    button { class: "toolbar-button toolbar-button--primary", r#type: "submit", "保存" }
                }
            }
            TableCell { "{field.field_type}" }
            TableCell { "{yes_no(field.is_required)}" }
            TableCell { "{field_expression(field)}" }
            TableCell {
                ActionForm { method: "post", action: ACTION_ENDPOINT,
                    HiddenInput { name: "action", value: "delete_field" }
                    HiddenInput { name: "model_name", value: model_name }
                    HiddenInput { name: "field_id", value: field.id.clone() }
                    button { class: "toolbar-button toolbar-button--danger", r#type: "submit", "删除" }
                }
            }
        }
    }
}

fn render_hooks(model_name: &str, hooks: &[HookDefinition]) -> Element {
    rsx! {
        div { class: "settings-section",
            ActionForm {
                method: "post",
                action: ACTION_ENDPOINT,
                class: "settings-form",
                HiddenInput { name: "action", value: "create_hook" }
                HiddenInput { name: "model_name", value: model_name }
                FormGrid { wide: true,
                    FormRow { label: "event".to_string(), required: true,
                        Select {
                            name: "trigger_event",
                            required: true,
                            options: hook_event_options("before_insert"),
                        }
                    }
                    FormRow { label: "order".to_string(),
                        Input { input_type: "number", name: "order_index", value: "0" }
                    }
                    FormRow { label: "script".to_string(), required: true, wide: true,
                        textarea {
                            class: "form-input settings-input",
                            name: "script_content",
                            rows: "5",
                            required: true,
                        }
                    }
                    CheckboxRow { name: "is_active", label: "启用".to_string(), checked: true }
                }
                button { class: "toolbar-button toolbar-button--primary", r#type: "submit", "添加钩子" }
            }
            TableViewport {
                Table { dense: true, striped: true, bordered: true,
                    TableHead {
                        TableRow {
                            TableHeaderCell { "事件" }
                            TableHeaderCell { "状态" }
                            TableHeaderCell { "顺序" }
                            TableHeaderCell { "脚本" }
                            TableHeaderCell { "操作" }
                        }
                    }
                    TableBody {
                        for hook in hooks {
                            {render_hook_edit_row(model_name, hook)}
                        }
                    }
                }
            }
        }
    }
}

fn render_hook_edit_row(model_name: &str, hook: &HookDefinition) -> Element {
    rsx! {
        TableRow {
            TableCell {
                ActionForm { method: "post", action: ACTION_ENDPOINT,
                    class: "settings-form",
                    HiddenInput { name: "action", value: "update_hook" }
                    HiddenInput { name: "model_name", value: model_name }
                    HiddenInput { name: "hook_id", value: hook.id.clone() }
                    FormGrid { wide: true,
                        FormRow { label: "event".to_string(), required: true,
                            Select {
                                name: "trigger_event",
                                required: true,
                                options: hook_event_options(&hook.trigger_event),
                            }
                        }
                        FormRow { label: "order".to_string(),
                            Input { input_type: "number", name: "order_index", value: hook.order_index.to_string() }
                        }
                        FormRow { label: "script".to_string(), required: true, wide: true,
                            textarea {
                                class: "form-input settings-input",
                                name: "script_content",
                                rows: "4",
                                required: true,
                                "{hook.script_content}"
                            }
                        }
                        CheckboxRow { name: "is_active", label: "启用".to_string(), checked: hook.is_active }
                    }
                    button { class: "toolbar-button toolbar-button--primary", r#type: "submit", "保存" }
                }
            }
            TableCell { "{active_label(hook.is_active)}" }
            TableCell { "{hook.order_index}" }
            TableCell { code { "{compact_script(&hook.script_content)}" } }
            TableCell {
                ActionForm { method: "post", action: ACTION_ENDPOINT,
                    HiddenInput { name: "action", value: "delete_hook" }
                    HiddenInput { name: "model_name", value: model_name }
                    HiddenInput { name: "hook_id", value: hook.id.clone() }
                    button { class: "toolbar-button toolbar-button--danger", r#type: "submit", "删除" }
                }
            }
        }
    }
}

fn render_records(
    model_name: &str,
    fields: &[MetaField],
    records: &[DataRecordView],
    total_records: u64,
) -> Element {
    rsx! {
        div { class: "settings-section",
            ActionForm {
                method: "post",
                action: ACTION_ENDPOINT,
                class: "settings-form",
                HiddenInput { name: "action", value: "create_record" }
                HiddenInput { name: "model_name", value: model_name }
                FormGrid { wide: true,
                    for field in fields.iter().filter(|field| field.field_type != "computed") {
                        {render_payload_field(field)}
                    }
                }
                button { class: "toolbar-button toolbar-button--primary", r#type: "submit", "插入记录" }
            }
            p { class: "lowcode-detail__subtitle", "total {total_records}" }
            TableViewport {
                Table { dense: true, striped: true, bordered: true,
                    TableHead {
                        TableRow {
                            TableHeaderCell { "id" }
                            for field in fields {
                                TableHeaderCell { "{field.display_name}" }
                            }
                            TableHeaderCell { "操作" }
                        }
                    }
                    TableBody {
                        for record in records {
                            {render_record_edit_row(model_name, fields, record)}
                        }
                    }
                }
            }
        }
    }
}

fn render_record_edit_row(
    model_name: &str,
    fields: &[MetaField],
    record: &DataRecordView,
) -> Element {
    let update_form_id = format!("record-update-{}", record.id);
    rsx! {
        TableRow {
            TableCell { code { "{record.id}" } }
            for field in fields {
                TableCell {
                    if field.field_type == "computed" {
                        "{payload_cell(&record.payload, &field.name)}"
                    } else {
                        {render_payload_input_for_form(field, &record.payload, &update_form_id)}
                    }
                }
            }
            TableCell {
                ActionForm { method: "post", action: ACTION_ENDPOINT, id: update_form_id.clone(), class: "toolbar",
                    HiddenInput { name: "action", value: "update_record" }
                    HiddenInput { name: "model_name", value: model_name }
                    HiddenInput { name: "record_id", value: record.id.clone() }
                    button { class: "toolbar-button toolbar-button--primary", r#type: "submit", "保存" }
                }
                ActionForm { method: "post", action: ACTION_ENDPOINT, class: "toolbar",
                    HiddenInput { name: "action", value: "delete_record" }
                    HiddenInput { name: "model_name", value: model_name }
                    HiddenInput { name: "record_id", value: record.id.clone() }
                    button { class: "toolbar-button toolbar-button--danger", r#type: "submit", "删除" }
                }
            }
        }
    }
}

fn render_payload_field(field: &MetaField) -> Element {
    let input_name = format!("payload_{}", field.name);
    let label = format!("{} · {}", field.display_name, field.field_type);
    let json_placeholder = "{}";
    match field.field_type.as_str() {
        "boolean" => rsx! {
            CheckboxRow { name: input_name, label }
        },
        "json" => rsx! {
            FormRow { label, required: field.is_required, wide: true,
                Input { name: input_name, required: field.is_required, placeholder: json_placeholder }
            }
        },
        "int" | "decimal" | "datetime" => rsx! {
            FormRow { label, required: field.is_required,
                Input { input_type: "number", name: input_name, required: field.is_required }
            }
        },
        _ => rsx! {
            FormRow { label, required: field.is_required,
                Input { name: input_name, required: field.is_required }
            }
        },
    }
}

fn render_payload_input_for_form(field: &MetaField, payload: &Value, form_id: &str) -> Element {
    let input_name = format!("payload_{}", field.name);
    let value = payload_input_value(payload, field);
    match field.field_type.as_str() {
        "boolean" => rsx! {
            input {
                form: form_id,
                r#type: "checkbox",
                name: input_name,
                value: "1",
                checked: payload_bool_value(payload, &field.name),
            }
        },
        "json" => rsx! {
            textarea {
                form: form_id,
                class: "form-input settings-input",
                name: input_name,
                rows: "2",
                required: field.is_required,
                "{value}"
            }
        },
        "int" | "decimal" | "datetime" => rsx! {
            input {
                form: form_id,
                class: "form-input settings-input",
                r#type: "number",
                name: input_name,
                value: value,
                required: field.is_required,
            }
        },
        _ => rsx! {
            input {
                form: form_id,
                class: "form-input settings-input",
                name: input_name,
                value: value,
                required: field.is_required,
            }
        },
    }
}

fn field_type_options(selected: &str) -> Vec<SelectOption> {
    [
        "string", "int", "decimal", "boolean", "datetime", "json", "computed",
    ]
    .into_iter()
    .map(|kind| SelectOption::new(kind, kind).selected(kind == selected))
    .collect()
}

fn hook_event_options(selected: &str) -> Vec<SelectOption> {
    [
        "before_insert",
        "before_update",
        "after_insert",
        "after_update",
    ]
    .into_iter()
    .map(|event| SelectOption::new(event, event).selected(event == selected))
    .collect()
}

fn tab_href(model_name: Option<&str>, tab: &str) -> String {
    match model_name {
        Some(model_name) => format!(
            "/?route=/lowcode&model={}&tab={tab}",
            urlencoding::encode(model_name)
        ),
        None => format!("/?route=/lowcode&tab={tab}"),
    }
}

fn parse_query_param(route: &str, key: &str) -> Option<String> {
    let query = route.split_once('?')?.1;
    for pair in query.split('&') {
        let (pair_key, pair_value) = match pair.split_once('=') {
            Some(value) => value,
            None => (pair, ""),
        };
        if pair_key == key {
            return Some(match urlencoding::decode(pair_value) {
                Ok(value) => value.into_owned(),
                Err(_) => pair_value.to_string(),
            });
        }
    }
    None
}

fn yes_no(value: bool) -> &'static str {
    if value { "是" } else { "否" }
}

fn active_label(value: bool) -> &'static str {
    if value { "active" } else { "inactive" }
}

fn optional_text(value: &Option<String>) -> String {
    value.clone().unwrap_or_default()
}

fn field_expression(field: &MetaField) -> String {
    match &field.expression {
        Some(value) => value.clone(),
        None => String::new(),
    }
}

fn payload_cell(payload: &Value, field_name: &str) -> String {
    let Some(value) = payload.get(field_name) else {
        return String::new();
    };
    match value {
        Value::String(value) => value.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn payload_input_value(payload: &Value, field: &MetaField) -> String {
    let Some(value) = payload.get(&field.name) else {
        return String::new();
    };
    match field.field_type.as_str() {
        "json" => match serde_json::to_string(value) {
            Ok(text) => text,
            Err(_) => String::new(),
        },
        _ => payload_cell(payload, &field.name),
    }
}

fn payload_bool_value(payload: &Value, field_name: &str) -> bool {
    payload
        .get(field_name)
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn compact_script(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(120)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_href_keeps_lowcode_route() {
        let href = tab_href(Some("order"), "hooks");

        // Admin UI 继续挂在 /lowcode，但不再进入旧页面渲染器。
        assert_eq!(href, "/?route=/lowcode&model=order&tab=hooks");
    }

    #[test]
    fn payload_cell_reads_json_payload() {
        let payload = serde_json::json!({ "amount": 99 });

        // 记录工作台展示 engine DataRecord.payload 字段。
        assert_eq!(payload_cell(&payload, "amount"), "99");
    }

    #[test]
    fn payload_input_keeps_json_value_editable() {
        let payload = serde_json::json!({ "meta": { "vip": true } });
        let field = MetaField {
            id: "field-meta".to_string(),
            model_name: "order".to_string(),
            name: "meta".to_string(),
            display_name: "元数据".to_string(),
            field_type: "json".to_string(),
            is_required: false,
            expression: None,
            dependency_json: None,
            order_index: 0,
            created_at_ms: 0,
            updated_at_ms: 0,
        };

        // JSON 字段编辑时必须保留合法 JSON 文本，提交后才能走真实解析。
        assert_eq!(payload_input_value(&payload, &field), r#"{"vip":true}"#);
    }

    #[test]
    fn ui_helpers_expose_four_workbench_blocks() {
        let field_href = tab_href(Some("order"), "fields");
        let hook_href = tab_href(Some("order"), "hooks");
        let record_href = tab_href(Some("order"), "records");

        // SSR 页面固定围绕模型树、字段、钩子、记录工作台组织。
        assert!(field_href.contains("tab=fields"));
        assert!(hook_href.contains("tab=hooks"));
        assert!(record_href.contains("tab=records"));
    }
}
