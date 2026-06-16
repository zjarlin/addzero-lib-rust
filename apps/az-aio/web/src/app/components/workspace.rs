#![allow(non_snake_case)]

use az_aio_platform::plugin::api::NativeRenderContext;
use az_dioxus_components::neobrutal::{
    NbContentSlot, NbFloatingPanelSlot, NbHeaderBar, NbIconButton, NbModelButton, NbProjectLayout,
    NbRightSlot, NbWorkspace, NbWorkspaceBody,
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

pub(super) fn Workspace(props: WorkspaceProps) -> Element {
    rsx! {
        NbWorkspace {
            HeaderBar {
                topbar_renderer: props.slots.topbar.clone(),
                render_context: props.render_context.clone(),
            }
            NbWorkspaceBody { lowcode: props.page.lowcode,
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
        NbHeaderBar {
            if let Some(render) = props.topbar_renderer {
                {render.render(props.render_context)}
            } else {
                NbModelButton {}
            }
            NbIconButton { id: "theme-toggle", href: "#", aria_label: "切换主题", "◐" }
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
    let has_plugin_renderer = props.renderer.is_some();

    rsx! {
        NbContentSlot { plugin: has_plugin_renderer,
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
            NbRightSlot {
                {render.render(props.render_context.clone())}
            }
        }

        if let Some(render) = props.slots.sandbox {
            NbFloatingPanelSlot {
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
        NbProjectLayout {
            if let Some(render) = props.project_sidebar {
                {render.render(props.render_context.clone())}
            }
            if let Some(render) = props.project_content {
                {render.render(props.render_context)}
            }
        }
    }
}
