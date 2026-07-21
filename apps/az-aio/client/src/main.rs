#![forbid(unsafe_code)]
#![allow(non_snake_case)]

use adui_dioxus::{Card, Content, Header, Layout, Sider, SiderTheme, ThemeProvider};
use dioxus::prelude::*;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use wasm_bindgen::JsValue;

const DEFAULT_ROUTE: &str = "/system/account/api-keys";
const MAX_LIST_ROWS: usize = 12;

fn main() {
    dioxus::LaunchBuilder::web()
        .with_cfg(dioxus_web::Config::new().rootname("aio-client-root"))
        .launch(App);
}

fn App() -> Element {
    let bootstrap = use_signal(load_bootstrap_from_document);
    let initial_route = initial_route(&bootstrap.read());
    let active_route = use_signal(move || initial_route);
    let route = active_route.read().clone();
    let snapshot = bootstrap.read().clone();
    let api_base_url = snapshot.api_base_url.clone();
    let title = page_title(&snapshot, &route);
    let content = render_client_page(&route, api_base_url);

    rsx! {
        ThemeProvider {
            Layout {
                has_sider: true,
                style: "min-height:100vh;",
                Sider {
                    width: 280.0,
                    theme: SiderTheme::Light,
                    style: "padding:16px 12px;overflow:auto;",
                    div { class: "adui-card-head-title", style: "padding:0 12px 16px;",
                        strong { "AZ AIO" }
                        div { class: "adui-typography-secondary", "Dioxus client plugin workbench" }
                    }
                    nav { class: "adui-menu adui-menu-inline", role: "menu",
                        ul { class: "adui-menu-list",
                            for section in snapshot.admin_menu_tree.sections.iter() {
                                li { class: "adui-menu-item adui-menu-submenu adui-menu-submenu-open",
                                    div { class: "adui-menu-item-title",
                                        span { class: "adui-menu-item-label", "{section.label}" }
                                    }
                                    ul { class: "adui-menu-submenu-list",
                                        for node in section.menus.iter() {
                                            {render_menu_node(node, &route, active_route)}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Layout {
                    Header {
                        style: "height:56px;display:flex;align-items:center;justify-content:space-between;padding:0 24px;border-bottom:1px solid var(--adui-color-border);",
                        strong { "{title}" }
                        code { class: "adui-tag", "{route}" }
                    }
                    Content {
                        style: "padding:24px;background:var(--adui-color-bg-layout);",
                        Card {
                            bordered: false,
                            style: "min-height:calc(100vh - 104px);",
                            {content}
                        }
                    }
                }
            }
        }
    }
}

fn render_menu_node(
    node: &AdminMenuNode,
    active_route: &str,
    mut active_route_signal: Signal<String>,
) -> Element {
    let active = menu_node_active(node, active_route);
    let class = if active {
        "adui-menu-item adui-menu-item-selected"
    } else {
        "adui-menu-item"
    };
    let href = if node.href.is_empty() {
        "#".to_string()
    } else {
        format!("?route={}", node.href)
    };
    let route_for_click = node.href.clone();

    rsx! {
        li { class, role: "menuitem",
            a {
                class: "adui-menu-item-title",
                href,
                style: "color:inherit;text-decoration:none;",
                onclick: move |event| {
                    if !route_for_click.is_empty() {
                        event.prevent_default();
                        active_route_signal.set(route_for_click.clone());
                        push_route(&route_for_click);
                    }
                },
                span { class: "adui-menu-item-icon", "{node.icon}" }
                span { class: "adui-menu-item-label", "{node.label}" }
            }
            if !node.children.is_empty() {
                ul { class: "adui-menu-submenu-list",
                    for child in node.children.iter() {
                        {render_menu_node(child, active_route, active_route_signal)}
                    }
                }
            }
        }
    }
}

fn render_client_page(route: &str, api_base_url: String) -> Element {
    match route {
        "/drive" => rsx! { DriveCenterClientPage { api_base_url } },
        "/software" => rsx! { SoftwareCenterClientPage { api_base_url } },
        "/assets" => rsx! { AssetHubClientPage { api_base_url } },
        _ => rsx! {
            Card {
                    h1 { "SSR fallback route" }
                    p { "当前路由尚未迁移到 Dioxus client，会继续由服务端 SSR fallback 提供首屏内容。" }
                    a { href: format!("?route={route}"), "Open SSR fallback" }
                }
        },
    }
}

#[component]
fn DriveCenterClientPage(api_base_url: String) -> Element {
    let snapshot = use_resource(move || {
        let api_base_url = api_base_url.clone();
        async move { load_drive_center_snapshot(&api_base_url).await }
    });
    render_resource(snapshot, drive_center_view)
}

#[component]
fn SoftwareCenterClientPage(api_base_url: String) -> Element {
    let snapshot = use_resource(move || {
        let api_base_url = api_base_url.clone();
        async move { load_software_center_snapshot(&api_base_url).await }
    });
    render_resource(snapshot, software_center_view)
}

#[component]
fn AssetHubClientPage(api_base_url: String) -> Element {
    let snapshot = use_resource(move || {
        let api_base_url = api_base_url.clone();
        async move { load_asset_hub_snapshot(&api_base_url).await }
    });
    render_resource(snapshot, asset_hub_view)
}

fn render_resource<T>(resource: Resource<Result<T, String>>, view: fn(&T) -> Element) -> Element
where
    T: 'static,
{
    match &*resource.read_unchecked() {
        Some(Ok(snapshot)) => view(snapshot),
        Some(Err(error)) => rsx! {
            Card {
                    h1 { "插件页面加载失败" }
                    p { class: "adui-alert-message", "{error}" }
                }
        },
        None => rsx! {
            Card {
                    h1 { "加载中" }
                    p { "正在通过插件 API 获取页面数据。" }
                }
        },
    }
}

fn drive_center_view(snapshot: &DriveCenterPageSnapshot) -> Element {
    let task_count = snapshot.tasks.len();
    rsx! {
        section { class: "adui-space adui-space-vertical",
            header { class: "adui-card-head",
                p { class: "adui-typography-secondary", "Operations / Storage" }
                h1 { "Drive Center" }
                p { "网盘任务、路径动作与 PostgreSQL 队列表。" }
            }
            div { class: "adui-row", style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:16px;",
                StatusCard { title: "运行态", status: snapshot.status.clone(), primary_api: "/api/drive-center/status" }
                Card {
                    h2 { "任务队列" }
                    p { "{task_count} 条来自 drive-center API 的任务记录。" }
                    if !snapshot.status.store_connected {
                        p { class: "adui-empty-description", "未连接数据库，当前不读取任务队列。" }
                    } else if snapshot.tasks.is_empty() {
                        p { class: "adui-empty-description", "数据库当前没有网盘任务。" }
                    } else {
                        table {
                            thead { tr { th { "路径" } th { "动作" } th { "状态" } th { "ID" } } }
                            tbody {
                                for task in snapshot.tasks.iter().take(MAX_LIST_ROWS) {
                                    tr {
                                        td { "{task.path}" }
                                        td { code { "{task.action}" } }
                                        td { "{task.status}" }
                                        td { code { "{task.id}" } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn software_center_view(snapshot: &SoftwareCenterPageSnapshot) -> Element {
    let installer_count = snapshot.installers.len();
    let package_count = snapshot.packages.len();
    rsx! {
        section { class: "adui-space adui-space-vertical",
            header { class: "adui-card-head",
                p { class: "adui-typography-secondary", "Knowledge / Software" }
                h1 { "Software Center" }
                p { "安装包扫描、归档结果与 PostgreSQL 软件包目录。" }
            }
            div { class: "adui-row", style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:16px;",
                StatusCard { title: "运行态", status: snapshot.status.clone(), primary_api: "/api/software-center/status" }
                Card {
                    h2 { "本机安装包" }
                    p { "{installer_count} 个文件来自插件扫描 API。" }
                    if snapshot.installers.is_empty() {
                        p { class: "adui-empty-description", "当前没有识别到安装包。" }
                    } else {
                        table {
                            thead { tr { th { "文件" } th { "平台" } th { "架构" } th { "状态" } } }
                            tbody {
                                for installer in snapshot.installers.iter().take(MAX_LIST_ROWS) {
                                    tr {
                                        td { "{installer.file_name}" }
                                        td { "{installer.platform}" }
                                        td { "{installer.arch}" }
                                        td { "{installer.status}" }
                                    }
                                }
                            }
                        }
                    }
                }
                Card {
                    h2 { "软件包目录" }
                    p { "{package_count} 条来自 software-center API 的软件包记录。" }
                    if !snapshot.status.store_connected {
                        p { class: "adui-empty-description", "未连接数据库，当前不读取软件包目录。" }
                    } else if snapshot.packages.is_empty() {
                        p { class: "adui-empty-description", "数据库当前没有软件包记录。" }
                    } else {
                        table {
                            thead { tr { th { "名称" } th { "平台" } th { "架构" } th { "状态" } } }
                            tbody {
                                for package in snapshot.packages.iter().take(MAX_LIST_ROWS) {
                                    tr {
                                        td { "{package.name}" }
                                        td { "{package.platform}" }
                                        td { "{package.arch}" }
                                        td { "{package.status}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn asset_hub_view(snapshot: &AssetHubPageSnapshot) -> Element {
    let asset_count = snapshot.assets.len();
    let skill_count = snapshot.scanned_skills.len();
    rsx! {
        section { class: "adui-space adui-space-vertical",
            header { class: "adui-card-head",
                p { class: "adui-typography-secondary", "Knowledge / Assets" }
                h1 { "Asset Hub" }
                p { "资产库、技能目录扫描与 PostgreSQL 持久化资产。" }
            }
            div { class: "adui-row", style: "display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:16px;",
                StatusCard { title: "运行态", status: snapshot.status.clone(), primary_api: "/api/asset-hub/status" }
                Card {
                    h2 { "持久化资产" }
                    p { "{asset_count} 条来自 asset-hub API 的资产记录。" }
                    if !snapshot.status.store_connected {
                        p { class: "adui-empty-description", "未连接数据库，当前不读取持久化资产。" }
                    } else if snapshot.assets.is_empty() {
                        p { class: "adui-empty-description", "数据库当前没有资产记录。" }
                    } else {
                        table {
                            thead { tr { th { "标题" } th { "类型" } th { "状态" } th { "来源" } } }
                            tbody {
                                for asset in snapshot.assets.iter().take(MAX_LIST_ROWS) {
                                    tr {
                                        td { "{asset.title}" }
                                        td { code { "{asset.kind}" } }
                                        td { "{asset.status}" }
                                        td { "{asset.source}" }
                                    }
                                }
                            }
                        }
                    }
                }
                Card {
                    h2 { "技能目录扫描" }
                    p { "{skill_count} 个技能来自 asset-hub API。" }
                    if snapshot.scanned_skills.is_empty() {
                        p { class: "adui-empty-description", "当前没有可展示的 SKILL.md 扫描结果。" }
                    } else {
                        ul {
                            for skill in snapshot.scanned_skills.iter().take(MAX_LIST_ROWS) {
                                li {
                                    strong { "{skill.name}" }
                                    span { " · {skill.status}" }
                                    br {}
                                    code { "{skill.source}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn StatusCard(title: &'static str, status: PluginStatus, primary_api: &'static str) -> Element {
    rsx! {
        Card {
            h2 { "{title}" }
            dl {
                dt { "状态接口" }
                dd { a { href: primary_api, "{primary_api}" } }
                dt { "DATABASE_URL" }
                dd { "{configured_text(status.database_configured)}" }
                dt { "数据表连接" }
                dd { "{connected_text(status.store_connected)}" }
                dt { "表前缀" }
                dd { code { "{status.table_prefix}" } }
            }
        }
    }
}

async fn load_drive_center_snapshot(api_base_url: &str) -> Result<DriveCenterPageSnapshot, String> {
    let status = fetch_json(api_base_url, "/api/drive-center/status").await?;
    let tasks = fetch_api_data(api_base_url, "/api/drive-center/tasks").await?;
    Ok(DriveCenterPageSnapshot { status, tasks })
}

async fn load_software_center_snapshot(
    api_base_url: &str,
) -> Result<SoftwareCenterPageSnapshot, String> {
    let status = fetch_json(api_base_url, "/api/software-center/status").await?;
    let installers = fetch_api_data(api_base_url, "/api/software-center/installers").await?;
    let packages = fetch_api_data(api_base_url, "/api/software-center/packages").await?;
    Ok(SoftwareCenterPageSnapshot {
        status,
        installers,
        packages,
    })
}

async fn load_asset_hub_snapshot(api_base_url: &str) -> Result<AssetHubPageSnapshot, String> {
    let status = fetch_json(api_base_url, "/api/asset-hub/status").await?;
    let scanned_skills = fetch_api_data(api_base_url, "/api/asset-hub/skills").await?;
    let assets = fetch_api_data(api_base_url, "/api/asset-hub/assets").await?;
    Ok(AssetHubPageSnapshot {
        status,
        assets,
        scanned_skills,
    })
}

async fn fetch_api_data<T>(api_base_url: &str, path: &str) -> Result<Vec<T>, String>
where
    T: DeserializeOwned,
{
    let response: ApiResponse<Vec<T>> = fetch_json(api_base_url, path).await?;
    if response.code == 200 {
        Ok(response.data.unwrap_or_default())
    } else {
        Err(response.msg)
    }
}

async fn fetch_json<T>(api_base_url: &str, path: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let url = api_url(api_base_url, path);
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|error| format!("GET {url} failed: {error}"))?;
    if !response.ok() {
        return Err(format!("GET {url} returned HTTP {}", response.status()));
    }
    response
        .json::<T>()
        .await
        .map_err(|error| format!("GET {url} returned invalid JSON: {error}"))
}

fn load_bootstrap_from_document() -> ClientBootstrapPayload {
    let json = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("aio-bootstrap"))
        .and_then(|element| element.text_content())
        .unwrap_or_default();
    serde_json::from_str(&json).unwrap_or_else(|_| ClientBootstrapPayload {
        admin_menu_tree: AdminMenuTree::default(),
        pages: Vec::new(),
        client_pages: Vec::new(),
        plugins: Vec::new(),
        default_route: DEFAULT_ROUTE.to_string(),
        api_base_url: String::new(),
    })
}

fn initial_route(bootstrap: &ClientBootstrapPayload) -> String {
    web_sys::window()
        .and_then(|window| window.location().search().ok())
        .and_then(|search| route_from_search(&search))
        .or_else(|| non_empty_route(&bootstrap.default_route))
        .or_else(|| first_client_route(bootstrap))
        .unwrap_or_else(|| bootstrap.default_route.clone())
}

fn non_empty_route(route: &str) -> Option<String> {
    (!route.is_empty()).then(|| route.to_string())
}

fn route_from_search(search: &str) -> Option<String> {
    search
        .trim_start_matches('?')
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find_map(|(key, value)| (key == "route").then(|| value.to_string()))
        .filter(|route| !route.is_empty())
}

fn first_client_route(bootstrap: &ClientBootstrapPayload) -> Option<String> {
    bootstrap
        .client_pages
        .first()
        .map(|page| page.route.clone())
}

fn page_title(bootstrap: &ClientBootstrapPayload, route: &str) -> String {
    bootstrap
        .pages
        .iter()
        .find(|page| page.route == route)
        .map(|page| page.title.clone())
        .or_else(|| {
            bootstrap
                .client_pages
                .iter()
                .find(|page| page.route == route)
                .map(|page| page.title.clone())
        })
        .unwrap_or_else(|| "Admin Workbench".to_string())
}

fn menu_node_active(node: &AdminMenuNode, active_route: &str) -> bool {
    node.href == active_route
        || node
            .active_patterns
            .iter()
            .any(|pattern| pattern == active_route)
        || node
            .children
            .iter()
            .any(|child| menu_node_active(child, active_route))
}

fn push_route(route: &str) {
    if let Some(window) = web_sys::window()
        && let Ok(history) = window.history()
    {
        let _ = history.push_state_with_url(&JsValue::NULL, "", Some(&format!("?route={route}")));
    }
}

fn api_url(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    if base.is_empty() {
        path.to_string()
    } else {
        format!("{base}{path}")
    }
}

fn configured_text(value: bool) -> &'static str {
    if value { "已配置" } else { "未配置" }
}

fn connected_text(value: bool) -> &'static str {
    if value { "已连接" } else { "未连接" }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
struct ClientBootstrapPayload {
    admin_menu_tree: AdminMenuTree,
    pages: Vec<PageContribution>,
    client_pages: Vec<ClientPageContribution>,
    plugins: Vec<ClientPluginRecord>,
    default_route: String,
    api_base_url: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
struct AdminMenuTree {
    #[serde(default)]
    sections: Vec<AdminMenuSection>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct AdminMenuSection {
    domain_id: String,
    label: String,
    default_href: String,
    order: i32,
    #[serde(default)]
    menus: Vec<AdminMenuNode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct AdminMenuNode {
    id: String,
    label: String,
    href: String,
    icon: String,
    order: i32,
    #[serde(default)]
    active_patterns: Vec<String>,
    #[serde(default)]
    children: Vec<AdminMenuNode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct PageContribution {
    route: String,
    title: String,
    subtitle: String,
    renderer_id: String,
    placeholder_mark: String,
    order: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct ClientPageContribution {
    route: String,
    title: String,
    renderer_id: String,
    order: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct ClientPluginRecord {
    descriptor: ClientPluginDescriptor,
    state: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct ClientPluginDescriptor {
    id: String,
    name: String,
    version: String,
    description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct ApiResponse<T> {
    code: u16,
    msg: String,
    data: Option<T>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct PluginStatus {
    ok: bool,
    database_configured: bool,
    store_connected: bool,
    table_prefix: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct DriveCenterPageSnapshot {
    status: PluginStatus,
    tasks: Vec<DriveTaskSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct DriveTaskSummary {
    id: String,
    path: String,
    action: String,
    status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct SoftwareCenterPageSnapshot {
    status: PluginStatus,
    installers: Vec<InstallerPackage>,
    packages: Vec<SoftwarePackageSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallerPackage {
    id: String,
    file_name: String,
    source_path: String,
    version: String,
    platform: String,
    arch: String,
    target_path: String,
    install_status: String,
    status: String,
    md5: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct SoftwarePackageSummary {
    id: String,
    name: String,
    source_path: String,
    platform: String,
    arch: String,
    status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct AssetHubPageSnapshot {
    status: PluginStatus,
    assets: Vec<AssetSummary>,
    scanned_skills: Vec<ScannedSkillAsset>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct AssetSummary {
    id: String,
    kind: String,
    title: String,
    status: String,
    source: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScannedSkillAsset {
    id: String,
    name: String,
    #[serde(rename = "type")]
    asset_type: String,
    source: String,
    origin: String,
    tags: Vec<String>,
    content: String,
    status: String,
    md5: Option<String>,
    systems: Vec<String>,
}
