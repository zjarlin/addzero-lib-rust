#![forbid(unsafe_code)]

use az_aio_plugin_host::host::HostSnapshot;

const DEFAULT_ROUTE: &str = "/assets";

pub fn render_app_html(snapshot: &HostSnapshot) -> String {
    let nav_html = render_nav(&snapshot.nav_items);
    let page = snapshot.pages.iter().find(|p| p.route == DEFAULT_ROUTE);
    let page_title = page
        .map(|p| p.title.as_str())
        .unwrap_or("No Plugin Renderer");
    let page_mark = page.map(|p| p.placeholder_mark.as_str()).unwrap_or("?");

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>AZ AIO</title>
    <link rel="stylesheet" href="/assets/app.css">
</head>
<body>
    <main class="az-aio-shell">
        <aside class="sidebar">
            <nav class="sidebar-tree sidebar-tree--primary">
                {nav_html}
            </nav>
        </aside>
        <section class="workspace">
            <header class="header-bar">
                <div class="header-bar__actions">
                    <button class="model-button" type="button">
                        <span class="model-button__mark">AZ</span>
                        <span>AZ AIO</span>
                    </button>
                </div>
            </header>
            <div class="workspace__body">
                <div class="empty-panel">
                    <div class="empty-panel__mark">{page_mark}</div>
                    <h1>{page_title}</h1>
                </div>
            </div>
        </section>
    </main>
</body>
</html>"#
    )
}

fn render_nav(items: &[az_aio_plugin_api::api::NavItemContribution]) -> String {
    items
        .iter()
        .map(|item| {
            format!(
                r#"<a class="sidebar-item" href="/?route={route}">
    <span class="sidebar-item__glyph">{icon}</span>
    <span class="sidebar-item__label">{label}</span>
</a>"#,
                route = item.route,
                icon = item.icon,
                label = item.label
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
