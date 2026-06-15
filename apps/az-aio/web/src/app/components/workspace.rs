#![allow(non_snake_case)]

use az_aio_platform::plugin::api::NativeRenderContext;
use dioxus::prelude::*;

use super::{
    model::{PageChrome, RenderSlot, SlotRenderers},
    welcome::WelcomeStart,
};

#[derive(PartialEq, Clone, Props)]
pub(super) struct WorkspaceProps {
    pub(super) slots: SlotRenderers,
    pub(super) page: PageChrome,
    pub(super) render_context: NativeRenderContext,
}

pub(super) fn Workspace(props: WorkspaceProps) -> Element {
    let body_class = if props.page.lowcode {
        "workspace__body workspace__body--lowcode"
    } else {
        "workspace__body"
    };

    rsx! {
        section { class: "workspace workbench-slot workbench-slot--main",
            HeaderBar {
                topbar_renderer: props.slots.topbar.clone(),
                render_context: props.render_context.clone(),
            }
            div { class: body_class,
                ContentSlot {
                    renderer: props.slots.content.clone(),
                    page: props.page,
                    render_context: props.render_context.clone(),
                }
                WorkspaceAddonSlots {
                    slots: props.slots,
                    render_context: props.render_context,
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct HeaderBarProps {
    topbar_renderer: Option<RenderSlot>,
    render_context: NativeRenderContext,
}

fn HeaderBar(props: HeaderBarProps) -> Element {
    rsx! {
        header { class: "header-bar",
            div { class: "header-bar__actions",
                if let Some(render) = props.topbar_renderer {
                    {render.render(props.render_context)}
                } else {
                    ModelButton {}
                }
                ThemeToggle {}
            }
        }
    }
}

fn ModelButton() -> Element {
    rsx! {
        button { class: "model-button", r#type: "button",
            span { class: "model-button__mark", "AZ" }
            span { "AZ AIO" }
            span { class: "model-button__chevron", "⌄" }
        }
    }
}

fn ThemeToggle() -> Element {
    rsx! {
        a {
            class: "icon-button",
            href: "#",
            id: "theme-toggle",
            "◐"
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct ContentSlotProps {
    renderer: Option<RenderSlot>,
    page: PageChrome,
    render_context: NativeRenderContext,
}

fn ContentSlot(props: ContentSlotProps) -> Element {
    let slot_class = if props.renderer.is_some() {
        "content-center-slot content-center-slot--plugin"
    } else {
        "content-center-slot content-center-slot--welcome"
    };

    rsx! {
        section { class: slot_class,
            if let Some(render) = props.renderer {
                {render.render(props.render_context)}
            } else {
                WelcomeStart { page: props.page }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct WorkspaceAddonSlotsProps {
    slots: SlotRenderers,
    render_context: NativeRenderContext,
}

fn WorkspaceAddonSlots(props: WorkspaceAddonSlotsProps) -> Element {
    let has_project_layout =
        props.slots.project_sidebar.is_some() || props.slots.project_content.is_some();

    rsx! {
        if has_project_layout {
            ProjectLayout {
                project_sidebar: props.slots.project_sidebar,
                project_content: props.slots.project_content,
                render_context: props.render_context.clone(),
            }
        }

        if let Some(render) = props.slots.settings {
            aside { class: "right-slot",
                {render.render(props.render_context.clone())}
            }
        }

        if let Some(render) = props.slots.sandbox {
            div { class: "floating-panel-slot",
                {render.render(props.render_context)}
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct ProjectLayoutProps {
    project_sidebar: Option<RenderSlot>,
    project_content: Option<RenderSlot>,
    render_context: NativeRenderContext,
}

fn ProjectLayout(props: ProjectLayoutProps) -> Element {
    rsx! {
        div { class: "project-layout",
            if let Some(render) = props.project_sidebar {
                {render.render(props.render_context.clone())}
            }
            if let Some(render) = props.project_content {
                {render.render(props.render_context)}
            }
        }
    }
}
