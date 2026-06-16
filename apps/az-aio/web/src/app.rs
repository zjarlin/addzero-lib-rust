#![forbid(unsafe_code)]

use az_aio_platform::plugin::api::{NavItemContribution, PageContribution};
use dioxus::prelude::*;

automod::dir!("src/app");

use components::AppLayout;

/// Render full HTML page.
pub fn render_app_html(
    snapshot: &az_aio_platform::plugin::host::HostSnapshot,
    route: &str,
    query: &str,
) -> String {
    let nav_items = snapshot_nav_items(snapshot);
    let pages = snapshot_pages(snapshot);

    let body = dioxus_ssr::render_element(rsx! {
        AppLayout {
            renderers: snapshot.native_renderers.clone(),
            nav_items,
            pages,
            route: route.to_string(),
            query: query.to_string(),
        }
    });

    format!(
        concat!(
            "<!DOCTYPE html>\n",
            "<html lang=\"zh-CN\" data-theme=\"light\">\n",
            "<head>\n",
            "    <meta charset=\"utf-8\">\n",
            "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
            "    <title>AZ AIO</title>\n",
            "    <link rel=\"stylesheet\" href=\"/assets/app.css\">\n",
            "</head>\n",
            "<body>\n",
            "    {body}\n",
            "    <script>(function(){{var r=document.documentElement;var saved=localStorage.getItem('az-theme');if(saved)r.setAttribute('data-theme',saved);var b=document.getElementById('theme-toggle');if(b)b.onclick=function(){{var t=r.getAttribute('data-theme')==='light'?'dark':'light';r.setAttribute('data-theme',t);localStorage.setItem('az-theme',t);return false;}};var shell=document.querySelector('.az-aio-shell');var sidebar=document.getElementById('sidebar-toggle');function setCollapsed(v){{if(!shell)return;shell.classList.toggle('az-aio-shell--collapsed',v);if(sidebar)sidebar.setAttribute('aria-expanded',String(!v));}}setCollapsed(localStorage.getItem('az-sidebar-collapsed')==='true');if(sidebar)sidebar.onclick=function(){{var next=!shell.classList.contains('az-aio-shell--collapsed');setCollapsed(next);localStorage.setItem('az-sidebar-collapsed',String(next));return false;}};}})();</script>\n",
            "</body>\n",
            "</html>",
        ),
        body = body,
    )
}

fn snapshot_nav_items(
    snapshot: &az_aio_platform::plugin::host::HostSnapshot,
) -> Vec<NavItemContribution> {
    if snapshot.nav_items.is_empty() {
        default_nav_items()
    } else {
        snapshot.nav_items.clone()
    }
}

fn snapshot_pages(snapshot: &az_aio_platform::plugin::host::HostSnapshot) -> Vec<PageContribution> {
    if snapshot.pages.is_empty() {
        default_pages()
    } else {
        snapshot.pages.clone()
    }
}

fn default_nav_items() -> Vec<NavItemContribution> {
    vec![
        nav("assets", "资产", "◆", "/assets", 10),
        nav("config", "配置", "⚙", "/config", 20),
        nav("gateway", "网关", "↗", "/gateway", 30),
        nav("software", "软件", "⬢", "/software", 40),
        nav("drive", "网盘", "⇄", "/drive", 50),
        nav("lowcode", "低代码", "▣", "/lowcode", 60),
    ]
}

fn default_pages() -> Vec<PageContribution> {
    vec![
        page(
            "/assets",
            "资产中心",
            "知识库 · 资产管理",
            "◆",
            "asset-hub.page",
            10,
        ),
        page(
            "/config",
            "配置中心",
            "配置管理",
            "⚙",
            "config-center.page",
            20,
        ),
        page(
            "/gateway",
            "边缘网关",
            "运维 · 网络",
            "↗",
            "edge-gateway.page",
            30,
        ),
        page(
            "/software",
            "软件中心",
            "安装器 · 目录",
            "⬢",
            "software-center.page",
            40,
        ),
        page("/drive", "网盘中心", "云存储", "⇄", "drive-center.page", 50),
        page(
            "/lowcode",
            "低代码工作台",
            "元数据建模 & AppScreen 低代码管理",
            "▣",
            "lowcode.page",
            60,
        ),
    ]
}

fn nav(id: &str, label: &str, icon: &str, route: &str, order: i32) -> NavItemContribution {
    NavItemContribution {
        id: id.into(),
        label: label.into(),
        icon: icon.into(),
        route: route.into(),
        order,
    }
}

fn page(
    route: &str,
    title: &str,
    subtitle: &str,
    mark: &str,
    renderer_id: &str,
    order: i32,
) -> PageContribution {
    PageContribution {
        route: route.into(),
        title: title.into(),
        subtitle: subtitle.into(),
        renderer_id: renderer_id.into(),
        placeholder_mark: mark.into(),
        order,
    }
}
