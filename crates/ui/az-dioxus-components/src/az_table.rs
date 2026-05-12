use dioxus::prelude::*;

use crate::class_name::compose_class;

/// Renders a semantic table root with `az-table` variants.
#[allow(non_snake_case)]
#[component]
pub fn AzTable(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] dense: bool,
    #[props(default)] striped: bool,
    #[props(default)] bordered: bool,
) -> Element {
    let table_class = compose_class(
        "az-table",
        &class,
        &[
            ("az-table--dense", dense),
            ("az-table--striped", striped),
            ("az-table--bordered", bordered),
        ],
    );

    rsx! {
        div { class: "az-table__scroller",
            table { class: table_class, {children} }
        }
    }
}

/// Renders a table caption with the `az-table__caption` class.
#[allow(non_snake_case)]
#[component]
pub fn AzTableCaption(children: Element, #[props(default, into)] class: String) -> Element {
    let caption_class = compose_class("az-table__caption", &class, &[]);

    rsx! {
        caption { class: caption_class, {children} }
    }
}

/// Renders the semantic table head section.
#[allow(non_snake_case)]
#[component]
pub fn AzTableHead(children: Element, #[props(default, into)] class: String) -> Element {
    let head_class = compose_class("az-table__head", &class, &[]);

    rsx! {
        thead { class: head_class, {children} }
    }
}

/// Renders the semantic table body section.
#[allow(non_snake_case)]
#[component]
pub fn AzTableBody(children: Element, #[props(default, into)] class: String) -> Element {
    let body_class = compose_class("az-table__body", &class, &[]);

    rsx! {
        tbody { class: body_class, {children} }
    }
}

/// Renders the semantic table footer section.
#[allow(non_snake_case)]
#[component]
pub fn AzTableFooter(children: Element, #[props(default, into)] class: String) -> Element {
    let footer_class = compose_class("az-table__footer", &class, &[]);

    rsx! {
        tfoot { class: footer_class, {children} }
    }
}

/// Renders a table row with an optional selected state class.
#[allow(non_snake_case)]
#[component]
pub fn AzTableRow(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] selected: bool,
) -> Element {
    let row_class = compose_class(
        "az-table__row",
        &class,
        &[("az-table__row--selected", selected)],
    );

    rsx! {
        tr { class: row_class, {children} }
    }
}

/// Renders a table header cell with a configurable scope.
#[allow(non_snake_case)]
#[component]
pub fn AzTableHeaderCell(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default = "col".to_string(), into)] scope: String,
    #[props(default)] numeric: bool,
) -> Element {
    let cell_class = compose_class(
        "az-table__header-cell",
        &class,
        &[("az-table__cell--numeric", numeric)],
    );

    rsx! {
        th { class: cell_class, scope: scope, {children} }
    }
}

/// Renders a table body or footer cell.
#[allow(non_snake_case)]
#[component]
pub fn AzTableCell(
    children: Element,
    #[props(default, into)] class: String,
    #[props(default)] numeric: bool,
    #[props(default = 1)] colspan: usize,
) -> Element {
    let cell_class = compose_class(
        "az-table__cell",
        &class,
        &[("az-table__cell--numeric", numeric)],
    );

    if colspan > 1 {
        rsx! {
            td { class: cell_class, colspan: "{colspan}", {children} }
        }
    } else {
        rsx! {
            td { class: cell_class, {children} }
        }
    }
}
