#![forbid(unsafe_code)]

use az_aio_platform::plugin_api::{NavItemContribution, NativeRenderContext, NativeUiRenderer, PageContribution, UiContributionSlot};
use dioxus::prelude::*;

#[derive(PartialEq, Clone, Props)]
struct ShellProps {
    renderers: Vec<NativeUiRenderer>,
    nav_items: Vec<NavItemContribution>,
    pages: Vec<PageContribution>,
    route: String,
    query: String,
}

/// Shell 插槽组件：侧栏 + 顶栏 + 内容区 + 设置区 + 项目区 + 沙箱面板。
#[allow(non_snake_case)]
fn AppLayout(props: ShellProps) -> Element {
    let api_base = String::new();
    let route = props.route.clone();
    let query = props.query.clone();

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
        active_route: format!("{}{}", route, query),
        api_base_url: api_base.clone(),
    };

    rsx! {
        main { class: "az-aio-shell",
            aside { class: "sidebar",
                if let Some(render) = sidebar_renderer {
                    {render(make_ctx())}
                } else {
                    nav { class: "sidebar-tree sidebar-tree--primary", style: "position:relative; z-index:1;",
                        for item in &props.nav_items {
                            a {
                                class: "nav-button",
                                href: "/?route={item.route}",
                                span { class: "nav-button__icon", "{item.icon}" }
                                span { class: "nav-button__label", "{item.label}" }
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
) -> Option<az_aio_platform::plugin_api::NativeRenderFn> {
    renderers
        .iter()
        .find(|r| r.slot == slot && r.route.as_deref() == Some(route))
        .map(|r| r.render)
}

/// Render full HTML page.
pub fn render_app_html(snapshot: &az_aio_platform::plugin_host::HostSnapshot, route: &str, query: &str) -> String {
    let body = dioxus_ssr::render_element(rsx! {
        AppLayout {
            renderers: snapshot.native_renderers.clone(),
            nav_items: if snapshot.nav_items.is_empty() { default_nav_items() } else { snapshot.nav_items.clone() },
            pages: if snapshot.pages.is_empty() { default_pages() } else { snapshot.pages.clone() },
            route: route.to_string(),
            query: query.to_string(),
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

fn default_nav_items() -> Vec<NavItemContribution> {
    vec![
        nav("assets", "Assets", "◆", "/assets", 10),
        nav("config", "Config", "⚙", "/config", 20),
        nav("gateway", "Gateway", "↗", "/gateway", 30),
        nav("software", "Software", "⬢", "/software", 40),
        nav("drive", "Drive", "⇄", "/drive", 50),
        nav("lowcode", "Lowcode", "▣", "/lowcode", 60),
    ]
}

fn default_pages() -> Vec<PageContribution> {
    vec![
        page("/assets", "Asset Hub", "Knowledge / Assets", "◆", "asset-hub.page", 10),
        page("/config", "Config Center", "Configuration Management", "⚙", "config-center.page", 20),
        page("/gateway", "Edge Gateway", "Operations / Network", "↗", "edge-gateway.page", 30),
        page("/software", "Software Center", "Installer & Catalog", "⬢", "software-center.page", 40),
        page("/drive", "Drive Center", "Cloud Drive", "⇄", "drive-center.page", 50),
    ]
}

fn nav(id: &str, label: &str, icon: &str, route: &str, order: i32) -> NavItemContribution {
    NavItemContribution { id: id.into(), label: label.into(), icon: icon.into(), route: route.into(), order }
}

fn page(route: &str, title: &str, subtitle: &str, mark: &str, renderer_id: &str, order: i32) -> PageContribution {
    PageContribution {
        route: route.into(), title: title.into(), subtitle: subtitle.into(),
        renderer_id: renderer_id.into(), placeholder_mark: mark.into(), order,
    }
}
