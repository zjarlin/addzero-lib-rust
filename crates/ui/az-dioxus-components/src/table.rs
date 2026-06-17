//! 遵循 `table-view` class 契约的表格基础组件。

use dioxus::prelude::*;

use crate::class_name::compose_class;

fn non_empty_attr(value: String) -> Option<String> {
    if value.is_empty() { None } else { Some(value) }
}

/// 渲染带有 `table-view` 变体 class 的语义化表格根节点。
#[allow(non_snake_case)]
#[component]
pub fn Table(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] dense: bool,
    #[props(default)] striped: bool,
    #[props(default)] bordered: bool,
    #[props(default = true)] frozen_header: bool,
) -> Element {
    let table_class = compose_class(
        "table-view",
        &class,
        &[
            ("table-view--dense", dense),
            ("table-view--striped", striped),
            ("table-view--bordered", bordered),
            ("table-view--frozen-header", frozen_header),
        ],
    );

    rsx! {
        div { class: "table-view__scroller",
            table { class: table_class, {children} }
        }
    }
}

/// 渲染带有 `table-view__caption` class 的表格标题。
#[allow(non_snake_case)]
#[component]
pub fn TableCaption(children: Element, #[props(default, into)] class: String) -> Element {
    let caption_class = compose_class("table-view__caption", &class, &[]);

    rsx! {
        caption { class: caption_class, {children} }
    }
}

/// 渲染语义化表头区域。
#[allow(non_snake_case)]
#[component]
pub fn TableHead(children: Element, #[props(default, into)] class: String) -> Element {
    let head_class = compose_class("table-view__head", &class, &[]);

    rsx! {
        thead { class: head_class, {children} }
    }
}

/// 渲染语义化表体区域。
#[allow(non_snake_case)]
#[component]
pub fn TableBody(children: Element, #[props(default, into)] class: String) -> Element {
    let body_class = compose_class("table-view__body", &class, &[]);

    rsx! {
        tbody { class: body_class, {children} }
    }
}

/// 渲染语义化表尾区域。
#[allow(non_snake_case)]
#[component]
pub fn TableFooter(children: Element, #[props(default, into)] class: String) -> Element {
    let footer_class = compose_class("table-view__footer", &class, &[]);

    rsx! {
        tfoot { class: footer_class, {children} }
    }
}

/// 渲染表格行，并可附加选中态 class。
#[allow(non_snake_case)]
#[component]
pub fn TableRow(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default, into)] style: String,
    #[props(default)] selected: bool,
) -> Element {
    let row_class = compose_class(
        "table-view__row",
        &class,
        &[("table-view__row--selected", selected)],
    );

    if style.is_empty() {
        rsx! {
            tr { class: row_class, {children} }
        }
    } else {
        rsx! {
            tr { class: row_class, style: style, {children} }
        }
    }
}

/// 渲染可配置 `scope` 的表头单元格。
#[allow(non_snake_case)]
#[component]
pub fn TableHeaderCell(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default, into)] style: String,
    #[props(default = "col".to_string(), into)] scope: String,
    #[props(default)] numeric: bool,
) -> Element {
    let cell_class = compose_class(
        "table-view__header-cell",
        &class,
        &[("table-view__cell--numeric", numeric)],
    );
    let style = non_empty_attr(style);

    rsx! {
        th { class: cell_class, style: style, scope: scope, {children} }
    }
}

/// 渲染表体或表尾单元格。
#[allow(non_snake_case)]
#[component]
pub fn TableCell(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default, into)] style: String,
    #[props(default)] numeric: bool,
    #[props(default = 1)] colspan: usize,
) -> Element {
    let cell_class = compose_class(
        "table-view__cell",
        &class,
        &[("table-view__cell--numeric", numeric)],
    );
    let style = non_empty_attr(style);

    if colspan > 1 {
        rsx! {
            td { class: cell_class, style: style, colspan: "{colspan}", {children} }
        }
    } else {
        rsx! {
            td { class: cell_class, style: style, {children} }
        }
    }
}
