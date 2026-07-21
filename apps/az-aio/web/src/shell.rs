use az_aio_platform::plugin::{
    api::{
        AdminMenuNode, AdminMenuNodeKind, AdminMenuSection, AdminResourceContract,
        NativeRenderContext, PageContribution,
    },
    host::{self, HostSnapshot},
};
use dioxus::prelude::*;
use registry::ui::{
    card::{Card, CardContent, CardDescription, CardHeader, CardTitle},
    table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow},
};

const FAVICON_DATA_URI: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'%3E%3Crect width='64' height='64' rx='14' fill='%230f172a'/%3E%3Ctext x='32' y='42' text-anchor='middle' font-size='26' font-family='Arial,sans-serif' font-weight='700' fill='%23ffffff'%3EA%3C/text%3E%3C/svg%3E";
const SHELL_STATE_SCRIPT: &str = r#"(() => {
  const scrollStorageKey = 'aio:scroll-state:v1';
  const sidebarStorageKey = 'aio:sidebar-collapsed:v1';
  const route = new URLSearchParams(window.location.search).get('route') || '/';
  const readScroll = () => {
    try { return JSON.parse(sessionStorage.getItem(scrollStorageKey) || '{}'); }
    catch (_) { return {}; }
  };
  const writeScroll = (state) => {
    try { sessionStorage.setItem(scrollStorageKey, JSON.stringify(state)); }
    catch (_) {}
  };
  const saveScroll = () => {
    const state = readScroll();
    const main = document.querySelector('[data-scroll-key="main"]');
    const sidebar = document.querySelector('[data-scroll-key="sidebar"]');
    state.main = state.main || {};
    if (main) state.main[route] = main.scrollTop;
    if (sidebar) state.sidebar = sidebar.scrollTop;
    writeScroll(state);
  };
  const restoreScroll = () => {
    const state = readScroll();
    const main = document.querySelector('[data-scroll-key="main"]');
    const sidebar = document.querySelector('[data-scroll-key="sidebar"]');
    if (main && state.main && Number.isFinite(state.main[route])) main.scrollTop = state.main[route];
    if (sidebar && Number.isFinite(state.sidebar)) sidebar.scrollTop = state.sidebar;
  };
  const shell = document.querySelector('[data-aio-shell]');
  const sidebarToggle = document.querySelector('[data-aio-sidebar-toggle]');
  const setSidebarCollapsed = (collapsed) => {
    if (!shell) return;
    shell.dataset.sidebarCollapsed = collapsed ? 'true' : 'false';
    if (sidebarToggle) {
      sidebarToggle.setAttribute('aria-expanded', collapsed ? 'false' : 'true');
      sidebarToggle.setAttribute('title', collapsed ? '展开侧边栏' : '收起侧边栏');
      sidebarToggle.setAttribute('aria-label', collapsed ? '展开侧边栏' : '收起侧边栏');
    }
  };
  const initialSidebarCollapsed = (() => {
    try { return localStorage.getItem(sidebarStorageKey) === 'true'; }
    catch (_) { return false; }
  })();
  setSidebarCollapsed(initialSidebarCollapsed);
  if (sidebarToggle) {
    sidebarToggle.addEventListener('click', () => {
      const next = shell?.dataset.sidebarCollapsed !== 'true';
      setSidebarCollapsed(next);
      try { localStorage.setItem(sidebarStorageKey, String(next)); }
      catch (_) {}
    });
  }
  if ('scrollRestoration' in history) history.scrollRestoration = 'manual';
  window.addEventListener('beforeunload', saveScroll);
  document.addEventListener('click', (event) => {
    const anchor = event.target && event.target.closest ? event.target.closest('a[href]') : null;
    if (anchor) saveScroll();
  }, true);
  requestAnimationFrame(restoreScroll);
})();"#;

/// Render the full SSR document with Dioxus components.
pub fn render_workbench_page(
    snapshot: &HostSnapshot,
    active_route: &str,
    api_base_url: &str,
) -> String {
    render_document(
        "AZ AIO",
        Some(render_bootstrap_json(snapshot, active_route, api_base_url)),
        workbench_shell(
            snapshot.clone(),
            active_route.to_string(),
            api_base_url.to_string(),
        ),
    )
}

/// Render a typed fallback document when the SSR task itself fails.
pub fn render_ssr_error_page(active_route: &str, error: &str) -> String {
    render_document(
        "AZ AIO SSR Error",
        None,
        error_body(active_route.to_string(), error.to_string()),
    )
}

pub fn route_matches(candidate: &str, active_route: &str) -> bool {
    if candidate.is_empty() {
        return false;
    }
    let (candidate_base, candidate_query) = split_route(candidate);
    let (active_base, active_query) = split_route(active_route);
    if candidate_base != active_base {
        return false;
    }
    match candidate_query {
        Some(query) => query_pairs_match(query, active_query.unwrap_or_default()),
        None => true,
    }
}

fn split_route(route: &str) -> (&str, Option<&str>) {
    match route.split_once('?') {
        Some((base, query)) => (base, Some(query)),
        None => (route, None),
    }
}

fn query_pairs_match(candidate_query: &str, active_query: &str) -> bool {
    candidate_query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .all(|pair| {
            let (candidate_key, candidate_value) = pair.split_once('=').unwrap_or((pair, ""));
            active_query.split('&').any(|active_pair| {
                let (active_key, active_value) =
                    active_pair.split_once('=').unwrap_or((active_pair, ""));
                active_key == candidate_key && active_value == candidate_value
            })
        })
}

fn render_document(title: &str, bootstrap_json: Option<String>, body: Element) -> String {
    let bootstrap_script = bootstrap_json
        .map(|json| {
            format!(r#"<script id="aio-bootstrap" type="application/json">{json}</script>"#)
        })
        .unwrap_or_default();
    let body = dioxus_ssr::render_element(body);

    format!(
        r#"<!DOCTYPE html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>{title}</title><link rel="icon" href="{FAVICON_DATA_URI}"><link rel="stylesheet" href="/assets/app.css?v=dictionary-dialog-v2"></head><body>{bootstrap_script}<div id="aio-root">{body}</div><script>{SHELL_STATE_SCRIPT}</script></body></html>"#
    )
}

fn error_body(active_route: String, error: String) -> Element {
    rsx! {
        main { class: "min-h-screen bg-background p-6 text-foreground",
            Card { class: "mx-auto max-w-3xl",
                CardHeader {
                    CardTitle { "Dioxus SSR 渲染失败" }
                    CardDescription {
                        "当前路由 " code { "{active_route}" } " 在服务端渲染时失败。"
                    }
                }
                CardContent {
                    pre { class: "overflow-auto rounded-md bg-muted p-4 text-sm", "{error}" }
                }
            }
        }
    }
}

fn workbench_shell(snapshot: HostSnapshot, active_route: String, api_base_url: String) -> Element {
    let active_page = active_page(&snapshot, &active_route).cloned();
    let title = active_page
        .as_ref()
        .map(|page| page.title.clone())
        .unwrap_or_else(|| "Admin Workbench".to_string());
    let current_href = route_href(&active_route);
    let sections = snapshot.admin_menu_tree.sections.clone();
    let active_section = active_menu_section(&sections, &active_route).cloned();
    let active_section_label = active_section
        .as_ref()
        .map(|section| section.label.clone())
        .unwrap_or_else(|| "导航".to_string());
    let active_menus = active_section
        .as_ref()
        .map(|section| section.menus.clone())
        .unwrap_or_default();

    rsx! {
        div { class: "aio-shell-frame bg-background text-foreground", "data-aio-shell": "true", "data-sidebar-collapsed": "false",
            aside { class: "aio-sidebar border-r bg-card",
                div { class: "aio-sidebar-header",
                    div { class: "aio-sidebar-header-row",
                        button {
                            class: "aio-icon-button aio-sidebar-toggle",
                            r#type: "button",
                            title: "收起侧边栏",
                            aria_label: "收起侧边栏",
                            aria_expanded: "true",
                            "data-aio-sidebar-toggle": "true",
                            svg {
                                class: "aio-sidebar-toggle-icon",
                                "aria-hidden": "true",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "1.8",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                rect { x: "3", y: "4", width: "18", height: "16", rx: "3" }
                                path { d: "M9 4v16" }
                            }
                        }
                        div { class: "aio-sidebar-brand min-w-0",
                            p { class: "aio-sidebar-brand-title truncate", "AZ AIO" }
                            p { class: "aio-sidebar-brand-subtitle truncate", "管理工作台" }
                        }
                    }
                }
                div { class: "aio-sidebar-scroll", "data-scroll-key": "sidebar",
                    {workbench_menu(active_menus, active_route.clone())}
                }
                footer { class: "aio-sidebar-footer text-xs text-muted-foreground",
                    span { class: "aio-sidebar-footer-icon", aria_hidden: "true", "◉" }
                    span { class: "aio-sidebar-footer-label truncate", "{active_section_label}" }
                }
            }
            main { class: "aio-main min-w-0",
                header { class: "aio-topbar border-b bg-background/95 backdrop-blur",
                    div { class: "aio-topbar-title min-w-0",
                        h1 { class: "truncate text-sm font-semibold", "{title}" }
                    }
                    {workbench_root_menu(sections, active_route.clone())}
                    div { class: "aio-toolbar-actions", aria_label: "Page tools",
                        a { class: "aio-icon-button", href: current_href, title: "重新加载当前页", aria_label: "重新加载当前页", "↻" }
                    }
                }
                section { class: "aio-main-scroll p-6", "data-scroll-key": "main",
                    {route_content(snapshot, active_route.clone(), api_base_url.clone())}
                }
            }
        }
    }
}

fn workbench_root_menu(sections: Vec<AdminMenuSection>, active_route: String) -> Element {
    rsx! {
        nav { class: "aio-root-menu", aria_label: "Root navigation",
            for section in sections {
                {root_menu_item(section, active_route.clone())}
            }
        }
    }
}

fn root_menu_item(section: AdminMenuSection, active_route: String) -> Element {
    let active = section_active(&section, &active_route);
    let href = section_href(&section);
    let class = if active {
        "aio-root-menu-item aio-root-menu-item--active"
    } else {
        "aio-root-menu-item"
    };

    rsx! {
        a { class, href,
            span { class: "truncate", "{section.label}" }
        }
    }
}

fn workbench_menu(menus: Vec<AdminMenuNode>, active_route: String) -> Element {
    if menus.is_empty() {
        return rsx! {
            p { class: "rounded-md border border-dashed p-4 text-sm text-muted-foreground",
                "当前根节点暂无菜单。"
            }
        };
    }

    rsx! {
        nav { class: "space-y-1", aria_label: "Side navigation",
            for node in menus {
                {menu_node(node, active_route.clone(), 0)}
            }
        }
    }
}

fn menu_node(node: AdminMenuNode, active_route: String, depth: usize) -> Element {
    let active = node_directly_active(&node, &active_route);
    let href = if node.href.is_empty() {
        "#".to_string()
    } else {
        route_href(&node.href)
    };
    let class = if active {
        "aio-sidebar-menu-link flex items-center gap-2 rounded-md bg-primary px-3 py-2 text-sm font-medium text-primary-foreground"
    } else {
        "aio-sidebar-menu-link flex items-center gap-2 rounded-md px-3 py-2 text-sm text-muted-foreground hover:bg-accent hover:text-accent-foreground"
    };
    let style = format!("padding-left:{}px", 12 + depth * 14);
    let icon = node.icon.clone();
    let label = node.label.clone();
    let children = node.children;

    rsx! {
        div { class: "aio-sidebar-menu-node space-y-1",
            a { class, style, href, title: label.clone(),
                span { class: "aio-sidebar-menu-icon w-4 shrink-0 text-center", "{icon}" }
                span { class: "aio-sidebar-menu-label min-w-0 truncate", "{label}" }
            }
            if !children.is_empty() {
                div { class: "aio-sidebar-menu-children space-y-1",
                    for child in children {
                        {menu_node(child, active_route.clone(), depth + 1)}
                    }
                }
            }
        }
    }
}

fn route_content(snapshot: HostSnapshot, active_route: String, api_base_url: String) -> Element {
    match active_page(&snapshot, &active_route).cloned() {
        Some(page) => {
            if let Some(renderer) = host::native_renderer(&snapshot, &page.renderer_id) {
                return renderer(NativeRenderContext {
                    active_route,
                    api_base_url,
                });
            }
            page_content(snapshot, page, active_route)
        }
        None => missing_route(snapshot.pages, active_route),
    }
}

fn page_content(snapshot: HostSnapshot, page: PageContribution, active_route: String) -> Element {
    let resource = snapshot
        .admin_resources
        .iter()
        .find(|resource| route_matches(&resource.route, &active_route))
        .cloned();

    if let Some(resource) = resource {
        return rsx! { {resource_card(resource)} };
    }

    rsx! {
        Card {
            CardHeader {
                CardTitle { "{page.title}" }
                CardDescription { "{page.subtitle}" }
            }
        }
    }
}

fn resource_card(resource: AdminResourceContract) -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { "{resource.label}" }
                CardDescription { "{resource.description}" }
            }
            CardContent {
                Table {
                    TableHeader {
                        TableRow {
                            TableHead { "Field" }
                            TableHead { "Kind" }
                            TableHead { "Required" }
                            TableHead { "Search" }
                        }
                    }
                    TableBody {
                        for field in resource.fields.iter() {
                            TableRow {
                                TableCell { class: "font-medium", "{field.label}" }
                                TableCell { class: "font-mono text-xs", "{field.kind:?}" }
                                TableCell { {yes_no(field.required)} }
                                TableCell { {yes_no(field.searchable)} }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn missing_route(pages: Vec<PageContribution>, active_route: String) -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { "未找到页面" }
                CardDescription {
                    "当前路由 " code { "{active_route}" } " 没有可用的 Dioxus 页面契约。"
                }
            }
            CardContent {
                ul { class: "space-y-2 text-sm",
                    for page in pages {
                        li {
                            a { class: "text-primary hover:underline", href: route_href(&page.route), "{page.title}" }
                            span { class: "ml-2 text-muted-foreground", "{page.subtitle}" }
                        }
                    }
                }
            }
        }
    }
}

fn active_menu_section<'a>(
    sections: &'a [AdminMenuSection],
    active_route: &str,
) -> Option<&'a AdminMenuSection> {
    sections
        .iter()
        .find(|section| section_active(section, active_route))
        .or_else(|| sections.first())
}

fn section_active(section: &AdminMenuSection, active_route: &str) -> bool {
    route_matches(&section.default_href, active_route)
        || section
            .menus
            .iter()
            .any(|node| node_active(node, active_route))
}

fn section_href(section: &AdminMenuSection) -> String {
    if !section.default_href.is_empty() {
        return route_href(&section.default_href);
    }
    section
        .menus
        .iter()
        .find_map(first_node_href)
        .map(|href| route_href(&href))
        .unwrap_or_else(|| "#".to_string())
}

fn first_node_href(node: &AdminMenuNode) -> Option<String> {
    if !node.href.is_empty() {
        return Some(node.href.clone());
    }
    node.children.iter().find_map(first_node_href)
}

fn route_href(route: &str) -> String {
    if route.is_empty() || route == "#" {
        "#".to_string()
    } else {
        format!("/?route={}", urlencoding::encode(route))
    }
}

fn active_page<'a>(snapshot: &'a HostSnapshot, active_route: &str) -> Option<&'a PageContribution> {
    snapshot
        .pages
        .iter()
        .find(|page| route_matches(&page.route, active_route))
}

fn node_active(node: &AdminMenuNode, active_route: &str) -> bool {
    node_directly_active(node, active_route)
        || node
            .children
            .iter()
            .any(|child| node_active(child, active_route))
}

fn node_directly_active(node: &AdminMenuNode, active_route: &str) -> bool {
    if node.kind == AdminMenuNodeKind::Branch && !node.children.is_empty() {
        return node.href == active_route
            || node
                .active_patterns
                .iter()
                .any(|pattern| pattern == active_route);
    }
    route_matches(&node.href, active_route)
        || node
            .active_patterns
            .iter()
            .any(|pattern| route_matches(pattern, active_route))
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn render_bootstrap_json(
    snapshot: &HostSnapshot,
    active_route: &str,
    api_base_url: &str,
) -> String {
    match serde_json::to_string(&host::client_bootstrap_payload(
        snapshot,
        active_route,
        api_base_url,
    )) {
        Ok(value) => escape_script_json(&value),
        Err(error) => {
            let fallback = serde_json::json!({
                "admin_menu_tree": { "sections": [] },
                "pages": [],
                "client_pages": [],
                "plugins": [],
                "default_route": "/",
                "api_base_url": "",
                "error": error.to_string(),
            });
            escape_script_json(&fallback.to_string())
        }
    }
}

fn escape_script_json(value: &str) -> String {
    value
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
}

#[cfg(test)]
mod tests {
    use az_aio_platform::plugin::{
        api::{AdminMenuNode, AdminMenuNodeKind, AdminMenuSection, PageContribution},
        host::HostSnapshot,
    };

    use super::*;

    #[test]
    fn dioxus_ui_shell_renders_contract_page_without_legacy_client_script() {
        let mut snapshot = HostSnapshot::default();
        snapshot.admin_menu_tree.sections.push(AdminMenuSection {
            domain_id: "demo".to_string(),
            label: "Demo".to_string(),
            default_href: "/demo".to_string(),
            order: 1,
            menus: Vec::new(),
        });
        snapshot.pages.push(PageContribution {
            route: "/demo".to_string(),
            title: "Demo Page".to_string(),
            subtitle: "Rendered by Dioxus".to_string(),
            renderer_id: "demo.page".to_string(),
            placeholder_mark: "D".to_string(),
            order: 1,
        });

        let html = render_workbench_page(&snapshot, "/demo", "");

        assert!(html.contains("管理工作台"));
        assert!(html.contains("Demo Page"));
        assert!(html.contains("data-name=\"Card\""));
        assert!(html.contains("data-aio-shell"));
        assert!(html.contains("data-aio-sidebar-toggle"));
        assert!(html.contains("aio:sidebar-collapsed:v1"));
        assert!(html.contains("aio-toolbar-actions"));
        assert!(!html.contains("az-aio-client.js"));
    }

    #[test]
    fn branch_context_does_not_visually_select_with_active_child() {
        let branch = AdminMenuNode {
            id: "engine.root".to_string(),
            kind: AdminMenuNodeKind::Branch,
            label: "低代码引擎".to_string(),
            href: "/lowcode".to_string(),
            icon: "▣".to_string(),
            order: 0,
            active_patterns: vec!["/lowcode".to_string()],
            permissions_any_of: Vec::new(),
            children: vec![AdminMenuNode {
                id: "engine.hooks".to_string(),
                kind: AdminMenuNodeKind::Page,
                label: "钩子".to_string(),
                href: "/lowcode?tab=hooks".to_string(),
                icon: "⚑".to_string(),
                order: 0,
                active_patterns: vec!["/lowcode?tab=hooks".to_string()],
                permissions_any_of: Vec::new(),
                children: Vec::new(),
            }],
        };
        let active_route = "/lowcode?tab=hooks";

        // 分支仍用于识别当前上下文，但黑色选中态只属于最具体的叶子菜单。
        assert!(node_active(&branch, active_route));
        assert!(!node_directly_active(&branch, active_route));
        assert!(node_directly_active(&branch.children[0], active_route));
    }
}
