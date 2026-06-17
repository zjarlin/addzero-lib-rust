#![allow(non_snake_case)]

use az_aio_platform::plugin::api::NativeRenderContext;
use az_dioxus_components::neobrutal::{
    ContentSlot as ContentSlotSurface, FloatingPanelSlot, HeaderBar as HeaderBarSurface,
    IconButton, ModelButton, ProjectLayout as ProjectLayoutSurface, RightSlot,
    Workspace as WorkspaceSurface, WorkspaceBody,
};
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

pub(super) fn WorkspaceChrome(props: WorkspaceProps) -> Element {
    rsx! {
        WorkspaceSurface {
            WorkspaceHeader {
                topbar_renderer: props.slots.topbar.clone(),
                render_context: props.render_context.clone(),
            }
            WorkspaceBody { lowcode: props.page.lowcode,
                WorkspaceContentSlot {
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

fn WorkspaceHeader(props: HeaderBarProps) -> Element {
    rsx! {
        HeaderBarSurface {
            if let Some(render) = props.topbar_renderer {
                {render.render(props.render_context)}
            } else {
                ModelButton {}
            }
            IconButton { id: "theme-toggle", href: "#", aria_label: "切换主题", "◐" }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct ContentSlotProps {
    renderer: Option<RenderSlot>,
    page: PageChrome,
    render_context: NativeRenderContext,
}

fn WorkspaceContentSlot(props: ContentSlotProps) -> Element {
    let has_plugin_renderer = props.renderer.is_some();

    rsx! {
        ContentSlotSurface { plugin: has_plugin_renderer,
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
            ProjectSlotsLayout {
                project_sidebar: props.slots.project_sidebar,
                project_content: props.slots.project_content,
                render_context: props.render_context.clone(),
            }
        }

        if let Some(render) = props.slots.settings {
            RightSlot {
                {render.render(props.render_context.clone())}
            }
        }

        if let Some(render) = props.slots.sandbox {
            FloatingPanelSlot {
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

fn ProjectSlotsLayout(props: ProjectLayoutProps) -> Element {
    rsx! {
        ProjectLayoutSurface {
            if let Some(render) = props.project_sidebar {
                {render.render(props.render_context.clone())}
            }
            if let Some(render) = props.project_content {
                {render.render(props.render_context)}
            }
        }
    }
}
