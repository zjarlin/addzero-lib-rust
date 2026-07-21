#![allow(non_snake_case)]

//! lowcode 插件的低代码引擎 Admin 页面。

use az_aio_platform::plugin::api::NativeRenderContext;
use az_engine::{DataRecordView, HookDefinition, MetaField, MetaModel, PageData, PageParams};
use dioxus::prelude::*;
use registry::ui::{
    badge::Badge,
    button::{Button, ButtonVariant},
    card::{Card, CardContent, CardDescription, CardHeader, CardTitle},
    table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow},
};
use serde_json::Value;

use crate::state::{run_engine_future, store};

const ACTION_ENDPOINT: &str = "/api/engine/ui-action";

#[derive(Clone, PartialEq)]
struct SelectOption {
    value: String,
    label: String,
    selected: bool,
}

impl SelectOption {
    fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            selected: false,
        }
    }

    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

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

/// 渲染低代码引擎的模型、字段、钩子和记录工作台。
pub fn LowcodePage(context: NativeRenderContext) -> Element {
    let snapshot = load_snapshot(&context.active_route);

    rsx! {
        div { class: "space-y-6",
            Card {
                CardHeader {
                    CardTitle { "低代码引擎" }
                    CardDescription { "模型、字段、钩子、记录统一管理；字段、钩子、记录入口统一放在左侧菜单。" }
                }
            }

            if let Some(error) = &snapshot.error {
                div { class: "rounded-xl border border-destructive bg-destructive/10 p-4 text-sm text-destructive", "{error}" }
            }

            div { class: "lowcode-workbench-grid grid gap-4",
                Card {
                    CardHeader {
                        CardTitle { "模型" }
                        CardDescription { "当前 {snapshot.models.len()} 个模型" }
                    }
                    CardContent { class: "space-y-4",
                        nav { class: "space-y-1",
                            for model in snapshot.models.iter() {
                                ModelLink {
                                    model: model.clone(),
                                    tab: snapshot.tab.clone(),
                                    active: snapshot.selected_model.as_deref() == Some(model.name.as_str()),
                                }
                            }
                        }
                        {render_model_form()}
                    }
                }

                div { class: "min-w-0",
                    if let Some(model_name) = snapshot.selected_model.as_deref() {
                        {render_selected_model(&snapshot, model_name)}
                    } else {
                        {render_empty_model_state()}
                    }
                }
            }
        }
    }
}

fn render_empty_model_state() -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { "选择或创建模型" }
                CardDescription { "创建第一个模型后即可维护字段、钩子和记录。" }
            }
            CardContent {
                Table {
                    TableHeader {
                        TableRow {
                            TableHead { "模型名" }
                            TableHead { "显示名" }
                            TableHead { "状态" }
                        }
                    }
                    TableBody {
                        TableRow {
                            TableCell { "—" }
                            TableCell { "暂无模型" }
                            TableCell { Badge { "empty" } }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ModelLink(model: MetaModel, tab: String, active: bool) -> Element {
    let class = if active {
        "flex items-center justify-between gap-3 rounded-md bg-primary px-3 py-2 text-sm text-primary-foreground"
    } else {
        "flex items-center justify-between gap-3 rounded-md px-3 py-2 text-sm text-muted-foreground hover:bg-accent hover:text-accent-foreground"
    };
    rsx! {
        a { class, href: tab_href(Some(&model.name), &tab),
            span { class: "min-w-0 truncate", "{model.display_name}" }
            Badge { class: "shrink-0 font-mono", "{model.name}" }
        }
    }
}

fn load_snapshot(route: &str) -> PageSnapshot {
    let selected_model = parse_query_param(route, "model");
    let tab = parse_query_param(route, "tab").unwrap_or_else(|| "fields".to_string());
    let mut error = parse_query_param(route, "error");
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

fn render_selected_model(snapshot: &PageSnapshot, model_name: &str) -> Element {
    let model = snapshot
        .models
        .iter()
        .find(|model| model.name == model_name)
        .cloned();
    let title = model
        .as_ref()
        .map(|model| model.display_name.clone())
        .unwrap_or_else(|| model_name.to_string());
    let display_name = model
        .map(|model| model.display_name)
        .unwrap_or_else(|| model_name.to_string());
    let subtitle = format!(
        "{} fields · {} hooks · {} records",
        snapshot.fields.len(),
        snapshot.hooks.len(),
        snapshot.total_records
    );

    rsx! {
        div { class: "space-y-4",
            Card {
                CardHeader {
                    div { class: "flex items-start justify-between gap-4",
                        div {
                            CardTitle { "{title}" }
                            CardDescription { "{subtitle}" }
                        }
                        div { class: "flex flex-wrap gap-2",
                            Badge { "{model_name}" }
                            Badge { "{snapshot.tab}" }
                        }
                    }
                }
                CardContent {
                    div { class: "grid gap-3 md:grid-cols-2",
                        ActionForm { method: "post", action: ACTION_ENDPOINT,
                            HiddenInput { name: "action", value: "update_model" }
                            HiddenInput { name: "model_name", value: model_name }
                            HiddenInput { name: "name", value: model_name }
                            div { class: "flex gap-2",
                                TextInput { name: "display_name", value: display_name, required: true, placeholder: "显示名" }
                                Button { button_type: "submit", "保存" }
                            }
                        }
                        ActionForm { method: "post", action: ACTION_ENDPOINT,
                            HiddenInput { name: "action", value: "delete_model" }
                            HiddenInput { name: "model_name", value: model_name }
                            Button { variant: ButtonVariant::Destructive, button_type: "submit", "删除模型" }
                        }
                    }
                }
            }

            if snapshot.tab == "hooks" {
                {render_hooks(model_name, &snapshot.hooks)}
            } else if snapshot.tab == "records" {
                {render_records(model_name, &snapshot.fields, &snapshot.records, snapshot.total_records)}
            } else {
                {render_fields(model_name, &snapshot.fields)}
            }
        }
    }
}

#[component]
fn ActionForm(
    method: &'static str,
    action: &'static str,
    children: Element,
    #[props(default, into)] class: String,
    #[props(into, optional)] id: Option<String>,
) -> Element {
    rsx! { form { id: id.as_deref(), class: format!("aio-form {class}"), method, action, {children} } }
}

#[component]
fn HiddenInput(name: &'static str, #[props(into)] value: String) -> Element {
    rsx! { input { r#type: "hidden", name, value } }
}

#[component]
fn TextInput(
    name: String,
    #[props(default = "text".to_string(), into)] input_type: String,
    #[props(default, into)] value: String,
    #[props(default, into)] placeholder: String,
    #[props(default)] required: bool,
    #[props(into, optional)] form: Option<String>,
) -> Element {
    rsx! {
        input {
            form: form.as_deref(),
            class: "aio-input",
            r#type: input_type,
            name,
            value,
            placeholder,
            required,
        }
    }
}

#[component]
fn SelectInput(
    name: &'static str,
    options: Vec<SelectOption>,
    #[props(default)] required: bool,
) -> Element {
    rsx! {
        select { class: "aio-input", name, required,
            for option in options {
                option { value: option.value, selected: option.selected, "{option.label}" }
            }
        }
    }
}

#[component]
fn CheckboxInput(name: String, label: String, #[props(default)] checked: bool) -> Element {
    rsx! {
        label { class: "inline-flex items-center gap-2 text-sm",
            input { r#type: "checkbox", name, value: "1", checked }
            span { "{label}" }
        }
    }
}

#[component]
fn FieldBlock(label: String, children: Element, #[props(default)] required: bool) -> Element {
    rsx! {
        label { class: "space-y-1 text-sm",
            span { class: "font-medium", "{label}" }
            {children}
            if required {
                span { class: "text-xs text-destructive", "必填" }
            }
        }
    }
}

fn render_model_form() -> Element {
    rsx! {
        ActionForm { method: "post", action: ACTION_ENDPOINT, class: "space-y-3",
            HiddenInput { name: "action", value: "create_model" }
            FieldBlock { label: "模型名".to_string(), required: true,
                TextInput { name: "name", required: true, placeholder: "order" }
            }
            FieldBlock { label: "显示名".to_string(), required: true,
                TextInput { name: "display_name", required: true, placeholder: "订单" }
            }
            Button { button_type: "submit", "新建模型" }
        }
    }
}

fn render_fields(model_name: &str, fields: &[MetaField]) -> Element {
    let dependency_placeholder = r#"[{"alias":"vip","source_model_name":"user","local_field":"user_id","source_payload_field":"vip"}]"#;
    rsx! {
        Card {
            CardHeader {
                CardTitle { "字段" }
                CardDescription { "定义字段类型、表达式和依赖。" }
            }
            CardContent { class: "space-y-4",
                ActionForm { method: "post", action: ACTION_ENDPOINT, class: "grid gap-3 md:grid-cols-2",
                    HiddenInput { name: "action", value: "create_field" }
                    HiddenInput { name: "model_name", value: model_name }
                    FieldBlock { label: "字段名".to_string(), required: true,
                        TextInput { name: "name", required: true, placeholder: "amount" }
                    }
                    FieldBlock { label: "显示名".to_string(), required: true,
                        TextInput { name: "display_name", required: true, placeholder: "金额" }
                    }
                    FieldBlock { label: "类型".to_string(), required: true,
                        SelectInput { name: "field_type", required: true, options: field_type_options("string") }
                    }
                    FieldBlock { label: "排序".to_string(),
                        TextInput { input_type: "number", name: "order_index", value: "0" }
                    }
                    FieldBlock { label: "表达式".to_string(),
                        TextInput { name: "expression", placeholder: "amount * 2" }
                    }
                    FieldBlock { label: "依赖 JSON".to_string(),
                        TextInput { name: "dependency_json", placeholder: dependency_placeholder }
                    }
                    CheckboxInput { name: "is_required", label: "必填".to_string() }
                    div { class: "flex items-end", Button { button_type: "submit", "添加字段" } }
                }

                Table {
                    TableHeader {
                        TableRow {
                            TableHead { "字段" }
                            TableHead { "类型" }
                            TableHead { "必填" }
                            TableHead { "表达式" }
                            TableHead { "操作" }
                        }
                    }
                    TableBody {
                        for field in fields {
                            {render_field_row(model_name, field)}
                        }
                    }
                }
            }
        }
    }
}

fn render_field_row(model_name: &str, field: &MetaField) -> Element {
    rsx! {
        TableRow {
            TableCell {
                ActionForm { method: "post", action: ACTION_ENDPOINT, class: "grid gap-2",
                    HiddenInput { name: "action", value: "update_field" }
                    HiddenInput { name: "model_name", value: model_name }
                    HiddenInput { name: "field_id", value: field.id.clone() }
                    TextInput { name: "name", value: field.name.clone(), required: true }
                    TextInput { name: "display_name", value: field.display_name.clone(), required: true }
                    SelectInput { name: "field_type", required: true, options: field_type_options(&field.field_type) }
                    TextInput { input_type: "number", name: "order_index", value: field.order_index.to_string() }
                    TextInput { name: "expression", value: optional_text(&field.expression) }
                    TextInput { name: "dependency_json", value: optional_text(&field.dependency_json) }
                    CheckboxInput { name: "is_required", label: "必填".to_string(), checked: field.is_required }
                    Button { button_type: "submit", "保存" }
                }
            }
            TableCell { Badge { "{field.field_type}" } }
            TableCell { "{yes_no(field.is_required)}" }
            TableCell { code { "{field_expression(field)}" } }
            TableCell {
                ActionForm { method: "post", action: ACTION_ENDPOINT,
                    HiddenInput { name: "action", value: "delete_field" }
                    HiddenInput { name: "model_name", value: model_name }
                    HiddenInput { name: "field_id", value: field.id.clone() }
                    Button { variant: ButtonVariant::Destructive, button_type: "submit", "删除" }
                }
            }
        }
    }
}

fn render_hooks(model_name: &str, hooks: &[HookDefinition]) -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { "钩子" }
                CardDescription { "维护 before/after 事件脚本。" }
            }
            CardContent { class: "space-y-4",
                ActionForm { method: "post", action: ACTION_ENDPOINT, class: "grid gap-3 md:grid-cols-2",
                    HiddenInput { name: "action", value: "create_hook" }
                    HiddenInput { name: "model_name", value: model_name }
                    FieldBlock { label: "事件".to_string(), required: true,
                        SelectInput { name: "trigger_event", required: true, options: hook_event_options("before_insert") }
                    }
                    FieldBlock { label: "排序".to_string(),
                        TextInput { input_type: "number", name: "order_index", value: "0" }
                    }
                    label { class: "space-y-1 text-sm md:col-span-2",
                        span { class: "font-medium", "脚本" }
                        textarea { class: "aio-input min-h-24", name: "script_content", required: true }
                    }
                    CheckboxInput { name: "is_active", label: "启用".to_string(), checked: true }
                    div { class: "flex items-end", Button { button_type: "submit", "添加钩子" } }
                }

                Table {
                    TableHeader {
                        TableRow {
                            TableHead { "事件" }
                            TableHead { "状态" }
                            TableHead { "顺序" }
                            TableHead { "脚本" }
                            TableHead { "操作" }
                        }
                    }
                    TableBody {
                        for hook in hooks {
                            {render_hook_row(model_name, hook)}
                        }
                    }
                }
            }
        }
    }
}

fn render_hook_row(model_name: &str, hook: &HookDefinition) -> Element {
    rsx! {
        TableRow {
            TableCell {
                ActionForm { method: "post", action: ACTION_ENDPOINT, class: "grid gap-2",
                    HiddenInput { name: "action", value: "update_hook" }
                    HiddenInput { name: "model_name", value: model_name }
                    HiddenInput { name: "hook_id", value: hook.id.clone() }
                    SelectInput { name: "trigger_event", required: true, options: hook_event_options(&hook.trigger_event) }
                    TextInput { input_type: "number", name: "order_index", value: hook.order_index.to_string() }
                    textarea { class: "aio-input min-h-20", name: "script_content", required: true, "{hook.script_content}" }
                    CheckboxInput { name: "is_active", label: "启用".to_string(), checked: hook.is_active }
                    Button { button_type: "submit", "保存" }
                }
            }
            TableCell { Badge { "{active_label(hook.is_active)}" } }
            TableCell { "{hook.order_index}" }
            TableCell { code { "{compact_script(&hook.script_content)}" } }
            TableCell {
                ActionForm { method: "post", action: ACTION_ENDPOINT,
                    HiddenInput { name: "action", value: "delete_hook" }
                    HiddenInput { name: "model_name", value: model_name }
                    HiddenInput { name: "hook_id", value: hook.id.clone() }
                    Button { variant: ButtonVariant::Destructive, button_type: "submit", "删除" }
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
        Card {
            CardHeader {
                CardTitle { "记录" }
                CardDescription { "共 {total_records} 条记录。" }
            }
            CardContent { class: "space-y-4",
                ActionForm { method: "post", action: ACTION_ENDPOINT, class: "grid gap-3 md:grid-cols-2",
                    HiddenInput { name: "action", value: "create_record" }
                    HiddenInput { name: "model_name", value: model_name }
                    for field in fields.iter().filter(|field| field.field_type != "computed") {
                        {render_payload_field(field)}
                    }
                    div { class: "flex items-end", Button { button_type: "submit", "插入记录" } }
                }

                Table {
                    TableHeader {
                        TableRow {
                            TableHead { "id" }
                            for field in fields {
                                TableHead { "{field.display_name}" }
                            }
                            TableHead { "操作" }
                        }
                    }
                    TableBody {
                        for record in records {
                            {render_record_row(model_name, fields, record)}
                        }
                    }
                }
            }
        }
    }
}

fn render_record_row(model_name: &str, fields: &[MetaField], record: &DataRecordView) -> Element {
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
                div { class: "flex gap-2",
                    ActionForm { method: "post", action: ACTION_ENDPOINT, id: update_form_id.clone(),
                        HiddenInput { name: "action", value: "update_record" }
                        HiddenInput { name: "model_name", value: model_name }
                        HiddenInput { name: "record_id", value: record.id.clone() }
                        Button { button_type: "submit", "保存" }
                    }
                    ActionForm { method: "post", action: ACTION_ENDPOINT,
                        HiddenInput { name: "action", value: "delete_record" }
                        HiddenInput { name: "model_name", value: model_name }
                        HiddenInput { name: "record_id", value: record.id.clone() }
                        Button { variant: ButtonVariant::Destructive, button_type: "submit", "删除" }
                    }
                }
            }
        }
    }
}

fn render_payload_field(field: &MetaField) -> Element {
    let input_name = format!("payload_{}", field.name);
    let label = format!("{} · {}", field.display_name, field.field_type);
    match field.field_type.as_str() {
        "boolean" => rsx! { CheckboxInput { name: input_name, label } },
        "json" => rsx! {
            label { class: "space-y-1 text-sm md:col-span-2",
                span { class: "font-medium", "{label}" }
                textarea { class: "aio-input min-h-20", name: input_name, required: field.is_required, placeholder: "{{}}" }
            }
        },
        "int" | "decimal" | "datetime" => rsx! {
            FieldBlock { label, required: field.is_required,
                TextInput { input_type: "number", name: input_name, required: field.is_required }
            }
        },
        _ => rsx! {
            FieldBlock { label, required: field.is_required,
                TextInput { name: input_name, required: field.is_required }
            }
        },
    }
}

fn render_payload_input_for_form(field: &MetaField, payload: &Value, form_id: &str) -> Element {
    let input_name = format!("payload_{}", field.name);
    let value = payload_input_value(payload, field);
    match field.field_type.as_str() {
        "boolean" => rsx! {
            input { form: form_id, r#type: "checkbox", name: input_name, value: "1", checked: payload_bool_value(payload, &field.name) }
        },
        "json" => rsx! {
            textarea { form: form_id, class: "aio-input min-h-16", name: input_name, required: field.is_required, "{value}" }
        },
        "int" | "decimal" | "datetime" => rsx! {
            input { form: form_id, class: "aio-input", r#type: "number", name: input_name, value, required: field.is_required }
        },
        _ => rsx! {
            input { form: form_id, class: "aio-input", name: input_name, value, required: field.is_required }
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
    let route = match model_name {
        Some(model_name) => format!(
            "/lowcode?model={}&tab={tab}",
            urlencoding::encode(model_name)
        ),
        None => format!("/lowcode?tab={tab}"),
    };
    format!("/?route={}", urlencoding::encode(&route))
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
        "json" => serde_json::to_string(value).unwrap_or_default(),
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

        // Admin UI 继续挂在 route=/lowcode?...，并通过侧轴菜单切换低代码工作台。
        assert_eq!(href, "/?route=%2Flowcode%3Fmodel%3Dorder%26tab%3Dhooks");
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
    fn ui_helpers_expose_three_detail_tabs() {
        let field_href = tab_href(Some("order"), "fields");
        let hook_href = tab_href(Some("order"), "hooks");
        let record_href = tab_href(Some("order"), "records");

        // 侧轴菜单链接必须把 /lowcode?... 整体编码进 route 参数，避免模型和 tab 丢失。
        assert!(field_href.contains("tab%3Dfields"));
        assert!(hook_href.contains("tab%3Dhooks"));
        assert!(record_href.contains("tab%3Drecords"));
    }
}
