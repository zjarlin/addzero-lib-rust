use az_aio_platform::plugin::api::{
    NativeRenderContext, NativeRenderFn, NativeUiRenderer, PageContribution, UiContributionSlot,
};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub(super) struct SlotRenderers {
    pub(super) content: Option<RenderSlot>,
    pub(super) settings: Option<RenderSlot>,
    pub(super) sidebar: Option<RenderSlot>,
    pub(super) topbar: Option<RenderSlot>,
    pub(super) project_sidebar: Option<RenderSlot>,
    pub(super) project_content: Option<RenderSlot>,
    pub(super) sandbox: Option<RenderSlot>,
}

impl SlotRenderers {
    pub(super) fn pick(renderers: &[NativeUiRenderer], route: &str) -> Self {
        Self {
            content: pick_renderer(renderers, UiContributionSlot::Content, route),
            settings: pick_renderer(renderers, UiContributionSlot::SettingsContent, route),
            sidebar: pick_renderer(renderers, UiContributionSlot::AppSidebar, route),
            topbar: pick_renderer(renderers, UiContributionSlot::AppTopbar, route),
            project_sidebar: pick_renderer(renderers, UiContributionSlot::ProjectSidebar, route),
            project_content: pick_renderer(renderers, UiContributionSlot::ProjectContent, route),
            sandbox: pick_renderer(renderers, UiContributionSlot::SandboxPanel, route),
        }
    }
}

#[derive(Clone)]
pub(super) struct RenderSlot {
    renderer_id: String,
    render: NativeRenderFn,
}

impl RenderSlot {
    pub(super) fn render(&self, context: NativeRenderContext) -> Element {
        (self.render)(context)
    }
}

impl PartialEq for RenderSlot {
    fn eq(&self, other: &Self) -> bool {
        self.renderer_id == other.renderer_id
    }
}

#[derive(Clone, PartialEq)]
pub(super) struct PageChrome {
    pub(super) title: String,
    pub(super) subtitle: String,
    pub(super) mark: String,
    pub(super) lowcode: bool,
}

impl PageChrome {
    pub(super) fn from_pages(pages: &[PageContribution], route: &str) -> Self {
        let page = pages.iter().find(|page| page.route == route);
        Self {
            title: page
                .map(|page| page.title.as_str())
                .unwrap_or("AZ AIO")
                .to_string(),
            subtitle: page
                .map(|page| page.subtitle.as_str())
                .unwrap_or("Plugin-first Rust full-stack workbench")
                .to_string(),
            mark: page
                .map(|page| page.placeholder_mark.as_str())
                .unwrap_or("AZ")
                .to_string(),
            lowcode: route.starts_with("/lowcode"),
        }
    }
}

fn pick_renderer(
    renderers: &[NativeUiRenderer],
    slot: UiContributionSlot,
    route: &str,
) -> Option<RenderSlot> {
    renderers
        .iter()
        .find(|renderer| {
            renderer.slot == slot && renderer_matches_route(renderer.route.as_deref(), route)
        })
        .map(|renderer| RenderSlot {
            renderer_id: renderer.renderer_id.clone(),
            render: renderer.render,
        })
}

fn renderer_matches_route(candidate: Option<&str>, route: &str) -> bool {
    candidate.is_none_or(|candidate| candidate == route)
}
