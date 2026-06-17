#![allow(non_snake_case)]

use az_aio_platform::plugin::api::{
    NativeRenderContext, NativeUiRenderer, NavItemContribution, PageContribution,
};
use az_dioxus_components::neobrutal::{
    NbShell, NbSidebarToggle, NbTitlebarControls, NbTitlebarNav,
};
use dioxus::prelude::*;

mod model;
mod sidebar;
mod welcome;
mod workspace;

use model::{PageChrome, SlotRenderers};
use sidebar::ShellSidebar;
use workspace::Workspace;

#[derive(PartialEq, Clone, Props)]
pub(crate) struct ShellProps {
    pub(crate) renderers: Vec<NativeUiRenderer>,
    pub(crate) nav_items: Vec<NavItemContribution>,
    pub(crate) pages: Vec<PageContribution>,
    pub(crate) route: String,
    pub(crate) query: String,
}

/// Pure workbench slot layout. The host places slots; plugins own the content.
pub(crate) fn AppLayout(props: ShellProps) -> Element {
    let route = props.route.clone();
    let slots = SlotRenderers::pick(&props.renderers, &route);
    let page = PageChrome::from_pages(&props.pages, &route);
    let active_route = format!("{}{}", props.route, props.query);
    let render_context = NativeRenderContext {
        active_route: active_route.clone(),
        api_base_url: String::new(),
    };

    rsx! {
        NbShell {
            TitlebarControls {}
            ShellSidebar {
                nav_items: props.nav_items.clone(),
                route: active_route,
                sidebar_renderer: slots.sidebar.clone(),
                render_context: render_context.clone(),
            }
            Workspace {
                slots,
                page,
                render_context,
            }
        }
    }
}

fn TitlebarControls() -> Element {
    rsx! {
        NbTitlebarControls {
            NbSidebarToggle { expanded: true }
            NbTitlebarNav { label: "‹" }
            NbTitlebarNav { label: "›", disabled: true }
        }
    }
}
