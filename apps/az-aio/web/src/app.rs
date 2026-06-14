#![forbid(unsafe_code)]

use az_aio_plugin_api::api::{NavItemContribution, NativeRenderContext, NativeUiRenderer, PageContribution, UiContributionSlot};
use dioxus::prelude::*;

const DEFAULT_ROUTE: &str = "/assets";

#[derive(PartialEq, Clone, Props)]
struct ShellProps {
    renderers: Vec<NativeUiRenderer>,
    nav_items: Vec<NavItemContribution>,
    pages: Vec<PageContribution>,
}

/// Shell 插槽组件：侧栏 + 顶栏 + 内容区 + 设置区 + 项目区 + 沙箱面板。
///
/// 通过 `renderers` 按 slot + route 匹配到插件注入的渲染器。
#[allow(non_snake_case)]
fn AzAioShell(props: ShellProps) -> Element {
    let api_base = "http://127.0.0.1:0".to_string();
    let route = DEFAULT_ROUTE.to_string();

    let content_renderer = pick_renderer(&props.renderers, UiContributionSlot::Content, &route);
    let settings_renderer = pick_renderer(&props.renderers, UiContributionSlot::SettingsContent, &route);
    let sidebar_renderer = pick_renderer(&props.renderers, UiContributionSlot::AppSidebar, &route);
    let topbar_renderer = pick_renderer(&props.renderers, UiContributionSlot::AppTopbar, &route);
    let project_sidebar = pick_renderer(&props.renderers, UiContributionSlot::ProjectSidebar, &route);
    let project_content = pick_renderer(&props.renderers, UiContributionSlot::ProjectContent, &route);
    let sandbox = pick_renderer(&props.renderers, UiContributionSlot::SandboxPanel, &route);

    let page = props.pages.iter().find(|p| p.route == route);
    let page_title = page.map(|p| p.title.clone()).unwrap_or_default();
    let page_mark = page.map(|p| p.placeholder_mark.clone()).unwrap_or_default();

    let make_ctx = || NativeRenderContext {
        active_route: route.clone(),
        api_base_url: api_base.clone(),
    };

    rsx! {
        main { class: "az-aio-shell",
            aside { class: "sidebar",
                if let Some(render) = sidebar_renderer {
                    {render(make_ctx())}
                } else {
                    nav { class: "sidebar-tree sidebar-tree--primary",
                        for item in &props.nav_items {
                            a {
                                class: "sidebar-item",
                                href: "/?route={item.route}",
                                span { class: "sidebar-item__glyph", "{item.icon}" }
                                span { class: "sidebar-item__label", "{item.label}" }
                            }
                        }
                    }
                }
            }
            section { class: "workspace",
                header { class: "header-bar",
                    div { class: "header-bar__actions",
                        if let Some(render) = topbar_renderer {
                            {render(make_ctx())}
                        } else {
                            button { class: "model-button", r#type: "button",
                                span { class: "model-button__mark", "AZ" }
                                span { "AZ AIO" }
                            }
                        }
                    }
                }
                div { class: "workspace__body",
                    if let Some(render) = content_renderer {
                        {render(make_ctx())}
                    } else {
                        div { class: "empty-panel",
                            div { class: "empty-panel__mark", "{page_mark}" }
                            h1 { "{page_title}" }
                        }
                    }

                    if project_sidebar.is_some() || project_content.is_some() {
                        div { class: "project-layout",
                            if let Some(render) = project_sidebar {
                                {render(make_ctx())}
                            }
                            if let Some(render) = project_content {
                                {render(make_ctx())}
                            }
                        }
                    }

                    if let Some(render) = settings_renderer {
                        {render(make_ctx())}
                    }

                    if let Some(render) = sandbox {
                        {render(make_ctx())}
                    }
                }
            }
        }
    }
}

fn pick_renderer(
    renderers: &[NativeUiRenderer],
    slot: UiContributionSlot,
    route: &str,
) -> Option<az_aio_plugin_api::api::NativeRenderFn> {
    renderers
        .iter()
        .find(|r| r.slot == slot && r.route.as_deref() == Some(route))
        .map(|r| r.render)
}

/// 渲染完整 HTML 页面。
pub fn render_app_html(snapshot: &az_aio_shared::state::HostSnapshot) -> String {
    let body = dioxus_ssr::render_element(rsx! {
        AzAioShell {
            renderers: snapshot.native_renderers.clone(),
            nav_items: snapshot.nav_items.clone(),
            pages: snapshot.pages.clone(),
        }
    });

    format!(
        concat!(
            "<!DOCTYPE html>\n",
            "<html lang=\"zh-CN\">\n",
            "<head>\n",
            "    <meta charset=\"utf-8\">\n",
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
            "    <title>AZ AIO</title>\n",
            "    <link rel=\"stylesheet\" href=\"/assets/app.css\">\n",
            "</head>\n",
            "<body>\n",
            "    {body}\n",
            "</body>\n",
            "</html>",
        ),
        body = body,
    )
}
