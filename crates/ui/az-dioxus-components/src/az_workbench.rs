//! Dense workbench layout primitives for admin-style pages.

use dioxus::prelude::*;

use crate::util::class_name::compose_class;

/// Full-height workbench page shell.
#[allow(non_snake_case)]
#[component]
pub fn AzWorkbenchPage(children: Element, #[props(default, into)] class: String) -> Element {
    let class = compose_class("az-workbench-page lowcode-page", &class, &[]);

    rsx! {
        section { class: class, {children} }
    }
}

/// Page header with title and subtitle.
#[allow(non_snake_case)]
#[component]
pub fn AzPageHeader(
    children: Element,
    #[props(into)] title: String,
    #[props(default, into)] subtitle: String,
    #[props(default, into)] class: String,
) -> Element {
    let class = compose_class("az-page-header lowcode-page__header", &class, &[]);

    rsx! {
        header { class: class,
            h1 { "{title}" }
            if !subtitle.is_empty() {
                p { "{subtitle}" }
            }
            {children}
        }
    }
}

/// Two-column split workbench shell, typically tree plus detail/table.
#[allow(non_snake_case)]
#[component]
pub fn AzSplitWorkbench(children: Element, #[props(default, into)] class: String) -> Element {
    let class = compose_class("az-split-workbench lowcode-workbench", &class, &[]);

    rsx! {
        div { class: class, {children} }
    }
}

/// Left tree panel in a split workbench.
#[allow(non_snake_case)]
#[component]
pub fn AzWorkbenchTree(children: Element, #[props(default, into)] class: String) -> Element {
    let class = compose_class("az-workbench-tree lowcode-tree", &class, &[]);

    rsx! {
        aside { class: class, {children} }
    }
}

/// Header row for a workbench tree panel.
#[allow(non_snake_case)]
#[component]
pub fn AzWorkbenchTreeHeader(
    children: Element,
    #[props(into)] title: String,
    #[props(default, into)] class: String,
) -> Element {
    let class = compose_class("az-workbench-tree__header lowcode-tree__header", &class, &[]);

    rsx! {
        div { class: class,
            h2 { "{title}" }
            {children}
        }
    }
}

/// Scrollable tree list area.
#[allow(non_snake_case)]
#[component]
pub fn AzWorkbenchTreeList(children: Element, #[props(default, into)] class: String) -> Element {
    let class = compose_class("az-workbench-tree__list lowcode-tree__list", &class, &[]);

    rsx! {
        div { class: class, {children} }
    }
}

/// Right detail panel in a split workbench.
#[allow(non_snake_case)]
#[component]
pub fn AzWorkbenchDetail(children: Element, #[props(default, into)] class: String) -> Element {
    let class = compose_class("az-workbench-detail lowcode-detail", &class, &[]);

    rsx! {
        section { class: class, {children} }
    }
}

/// Header row for a detail panel.
#[allow(non_snake_case)]
#[component]
pub fn AzWorkbenchDetailHeader(
    children: Element,
    #[props(into)] title: String,
    #[props(default, into)] subtitle: String,
    #[props(default, into)] class: String,
) -> Element {
    let class = compose_class("az-workbench-detail__header lowcode-detail__header", &class, &[]);

    rsx! {
        div { class: class,
            h2 { "{title}" }
            if !subtitle.is_empty() {
                p { class: "lowcode-detail__subtitle", "{subtitle}" }
            }
            {children}
        }
    }
}

/// Toolbar container for compact actions.
#[allow(non_snake_case)]
#[component]
pub fn AzToolbar(children: Element, #[props(default, into)] class: String) -> Element {
    let class = compose_class("az-toolbar", &class, &[]);

    rsx! {
        div { class: class, {children} }
    }
}

/// Scroll container for dense table areas.
#[allow(non_snake_case)]
#[component]
pub fn AzTableViewport(children: Element, #[props(default, into)] class: String) -> Element {
    let class = compose_class("az-table-viewport lowcode-table-scroll", &class, &[]);

    rsx! {
        div { class: class, {children} }
    }
}
