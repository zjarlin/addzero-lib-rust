#![allow(non_snake_case)]

use az_aio_platform::plugin_api::{
    NativeRenderContext, NativeUiRenderer, NavItemContribution, PageContribution,
};
use dioxus::prelude::*;

automod::dir!("src/app/components");

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
    let render_context = NativeRenderContext {
        active_route: format!("{}{}", props.route, props.query),
        api_base_url: String::new(),
    };

    rsx! {
        main { class: "az-aio-shell",
            TitlebarControls {}
            ShellSidebar {
                nav_items: props.nav_items.clone(),
                route: route.clone(),
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
        div { class: "titlebar-controls",
            button {
                class: "sidebar-toggle",
                id: "sidebar-toggle",
                r#type: "button",
                "aria-label": "折叠侧边栏",
                "aria-expanded": "true",
                span { class: "sidebar-toggle__glyph" }
            }
            span { class: "titlebar-nav", "aria-hidden": "true", "‹" }
            span { class: "titlebar-nav titlebar-nav--disabled", "aria-hidden": "true", "›" }
        }
    }
}
