//! Dense workbench layout primitives for admin-style pages.

use dioxus::prelude::*;

use crate::class_name::compose_class;

/// Full-height workbench page shell.
#[allow(non_snake_case)]
#[component]
pub fn WorkbenchPage(children: Element, #[props(default, into)] class: String) -> Element {
    let class = compose_class("workbench-page lowcode-page", &class, &[]);

    rsx! {
        section { class: class, {children} }
    }
}

/// Page header with title and subtitle.
#[allow(non_snake_case)]
#[component]
pub fn PageHeader(
    children: Element,
    #[props(into)] title: String,
    #[props(default, into)] subtitle: String,
    #[props(default, into)] class: String,
) -> Element {
    let class = compose_class("page-header lowcode-page__header", &class, &[]);

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
pub fn SplitWorkbench(children: Element, #[props(default, into)] class: String) -> Element {
    let class = compose_class("split-workbench lowcode-workbench", &class, &[]);

    rsx! {
        div { class: class, {children} }
    }
}

/// Left tree panel in a split workbench.
#[allow(non_snake_case)]
#[component]
pub fn WorkbenchTree(children: Element, #[props(default, into)] class: String) -> Element {
    let class = compose_class("workbench-tree lowcode-tree", &class, &[]);

    rsx! {
        aside { class: class, {children} }
    }
}

/// Header row for a workbench tree panel.
#[allow(non_snake_case)]
#[component]
pub fn WorkbenchTreeHeader(
    children: Element,
    #[props(into)] title: String,
    #[props(default, into)] class: String,
) -> Element {
    let class = compose_class("workbench-tree__header lowcode-tree__header", &class, &[]);

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
pub fn WorkbenchTreeList(children: Element, #[props(default, into)] class: String) -> Element {
    let class = compose_class("workbench-tree__list lowcode-tree__list", &class, &[]);

    rsx! {
        div { class: class, {children} }
    }
}

/// Right detail panel in a split workbench.
#[allow(non_snake_case)]
#[component]
pub fn WorkbenchDetail(children: Element, #[props(default, into)] class: String) -> Element {
    let class = compose_class("workbench-detail lowcode-detail", &class, &[]);

    rsx! {
        section { class: class, {children} }
    }
}

/// Header row for a detail panel.
#[allow(non_snake_case)]
#[component]
pub fn WorkbenchDetailHeader(
    children: Element,
    #[props(into)] title: String,
    #[props(default, into)] subtitle: String,
    #[props(default, into)] class: String,
) -> Element {
    let class = compose_class(
        "workbench-detail__header lowcode-detail__header",
        &class,
        &[],
    );

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
pub fn Toolbar(children: Element, #[props(default, into)] class: String) -> Element {
    let class = compose_class("toolbar", &class, &[]);

    rsx! {
        div { class: class, {children} }
    }
}

/// Scroll container for dense table areas.
#[allow(non_snake_case)]
#[component]
pub fn TableViewport(children: Element, #[props(default, into)] class: String) -> Element {
    let class = compose_class("table-view-viewport lowcode-table-scroll", &class, &[]);

    rsx! {
        div { class: class, {children} }
    }
}
