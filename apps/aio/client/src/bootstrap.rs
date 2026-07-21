use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

const DEFAULT_ROUTE: &str = "/system/account/api-keys";

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientBootstrapPayload {
    pub admin_menu_tree: AdminMenuTree,
    pub pages: Vec<PageContribution>,
    pub client_pages: Vec<ClientPageContribution>,
    pub plugins: Vec<ClientPluginRecord>,
    pub default_route: String,
    pub api_base_url: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdminMenuTree {
    #[serde(default)]
    pub sections: Vec<AdminMenuSection>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdminMenuSection {
    pub domain_id: String,
    pub label: String,
    pub default_href: String,
    pub order: i32,
    #[serde(default)]
    pub menus: Vec<AdminMenuNode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdminMenuNode {
    pub id: String,
    pub label: String,
    pub href: String,
    pub icon: String,
    pub order: i32,
    #[serde(default)]
    pub active_patterns: Vec<String>,
    #[serde(default)]
    pub children: Vec<AdminMenuNode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PageContribution {
    pub route: String,
    pub title: String,
    pub subtitle: String,
    pub renderer_id: String,
    pub placeholder_mark: String,
    pub order: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientPageContribution {
    pub route: String,
    pub title: String,
    pub renderer_id: String,
    pub order: i32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientPluginRecord {
    pub descriptor: ClientPluginDescriptor,
    pub state: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientPluginDescriptor {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub msg: String,
    pub data: Option<T>,
}

pub fn load_from_document() -> ClientBootstrapPayload {
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

pub fn initial_route(bootstrap: &ClientBootstrapPayload) -> String {
    web_sys::window()
        .and_then(|window| window.location().search().ok())
        .and_then(|search| route_from_search(&search))
        .or_else(|| non_empty_route(&bootstrap.default_route))
        .or_else(|| first_client_route(bootstrap))
        .unwrap_or_else(|| bootstrap.default_route.clone())
}

pub fn page_title(bootstrap: &ClientBootstrapPayload, route: &str) -> String {
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

pub fn menu_node_active(node: &AdminMenuNode, active_route: &str) -> bool {
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

pub fn push_route(route: &str) {
    if let Some(window) = web_sys::window()
        && let Ok(history) = window.history()
    {
        let _ = history.push_state_with_url(&JsValue::NULL, "", Some(&format!("?route={route}")));
    }
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
