//! 系统后台 Dioxus SSR 页面。

#![allow(non_snake_case)]

use crate::{
    plugin::api::NativeRenderContext,
    system::{
        catalog::{
            SYSTEM_DEFAULT_ROUTE, SystemDashboardView, SystemOperation, SystemPageView,
            SystemTableCell, system_dashboard_view, system_page_for_route,
        },
        navigation::{AdminNodeSnapshot, system_admin_sections},
    },
};
use az_dioxus_components::neobrutal::{
    Badge, BlockTitle, Card, Eyebrow, Grid, Hero, Page, Split,
};
use dioxus::prelude::*;

const SYSTEM_ADMIN_STYLE: &str = r#"
.system-admin-page {
  --page-bg: #f7efe6;
  --page-main: #e57c3a;
  --page-accent: #67b7dc;
  --page-secondary: #f6d64a;
  --page-shadow: 4px 4px 0 var(--page-line);
  --page-radius: 5px;
  background: var(--page-bg);
}

.system-admin-hero {
  min-height: 154px;
  grid-template-columns: minmax(0, 1fr) minmax(240px, 340px);
  background: #ffffff;
  background-image: none;
}

.system-admin-hero h1 {
  font-size: clamp(30px, 4vw, 48px);
  letter-spacing: 0;
}

.system-admin-hero__stats {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 10px;
}

.system-admin-stat {
  min-height: 78px;
  display: grid;
  align-content: center;
  gap: 6px;
  padding: 12px;
  border: 2px solid var(--page-line);
  border-radius: 5px;
  background: var(--page-secondary);
  box-shadow: 4px 4px 0 var(--page-line);
}

.system-admin-stat:nth-child(2) {
  background: var(--page-accent);
}

.system-admin-stat:nth-child(3) {
  background: var(--page-main);
}

.system-admin-stat span {
  color: var(--page-muted);
  font-size: 12px;
  font-weight: 800;
}

.system-admin-stat strong {
  color: var(--page-ink);
  font-size: 28px;
  font-weight: 900;
  line-height: 1;
}

.system-admin-layout {
  grid-template-columns: minmax(0, 1fr) minmax(300px, 360px);
}

.system-admin-main {
  min-width: 0;
  display: grid;
  gap: 16px;
}

.system-admin-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  background: var(--page-panel);
}

.system-admin-toolbar__title,
.system-admin-toolbar__actions {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 10px;
}

.system-admin-toolbar__actions {
  flex-wrap: wrap;
  justify-content: flex-end;
}

.system-admin-action-form {
  display: inline-flex;
  margin: 0;
}

.system-admin-command {
  min-height: 38px;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  border: 2px solid var(--page-line);
  border-radius: 5px;
  background: #ffffff;
  box-shadow: 4px 4px 0 var(--page-line);
  color: var(--page-ink);
  font: inherit;
  font-size: 12px;
  font-weight: 900;
  cursor: default;
}

.system-admin-command--primary {
  background: var(--page-accent);
  color: #ffffff;
}

.system-admin-toolbar h2 {
  margin: 0;
  color: var(--page-ink);
  font-size: 20px;
  font-weight: 900;
}

.system-admin-toolbar p {
  margin: 4px 0 0;
  color: var(--page-muted);
  font-size: 12px;
  font-weight: 800;
}

.system-admin-mark {
  width: 42px;
  height: 42px;
  display: inline-grid;
  place-items: center;
  flex: 0 0 auto;
  border: 2px solid var(--page-line);
  border-radius: 5px;
  background: var(--page-main);
  box-shadow: 4px 4px 0 var(--page-line);
  color: #111111;
  font-size: 20px;
  font-weight: 900;
}

.system-admin-table-card {
  display: grid;
  gap: 14px;
}

.system-admin-querybar {
  display: grid;
  grid-template-columns: minmax(220px, 1fr) auto auto;
  gap: 10px;
  align-items: end;
}

.system-admin-search {
  min-width: 0;
  display: grid;
  gap: 6px;
  color: var(--page-ink);
  font-size: 12px;
  font-weight: 900;
}

.system-admin-search input {
  min-width: 0;
  height: 38px;
  padding: 8px 10px;
  border: 2px solid var(--page-line);
  border-radius: 5px;
  background: #ffffff;
  box-shadow: 4px 4px 0 var(--page-line);
  color: var(--page-ink);
  font: inherit;
  font-size: 13px;
  font-weight: 750;
  transition:
    transform 150ms ease,
    box-shadow 150ms ease;
}

.system-admin-search input:focus {
  outline: 2px solid #ffffff;
  outline-offset: 2px;
  transform: translate(4px, 4px);
  box-shadow: none;
}

.system-admin-pagination,
.system-admin-store-link,
.system-admin-store-block a,
.system-admin-store-chip {
  min-height: 38px;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  border: 2px solid var(--page-line);
  border-radius: 5px;
  background: #ffffff;
  box-shadow: 4px 4px 0 var(--page-line);
  color: var(--page-ink);
  font-size: 12px;
  font-weight: 900;
  text-decoration: none;
  white-space: nowrap;
}

.system-admin-pagination a,
.system-admin-store-link,
.system-admin-store-block a,
.system-admin-store-chip {
  color: var(--page-ink);
}

.system-admin-pagination a:hover,
.system-admin-store-link:hover,
.system-admin-store-block a:hover {
  transform: translate(4px, 4px);
  box-shadow: none;
}

.system-admin-store-link--static,
.system-admin-store-chip {
  cursor: default;
}

.system-admin-table-scroll {
  min-width: 0;
  overflow: auto;
  border: 2px solid var(--page-line);
  border-radius: 5px;
  background: #ffffff;
}

.system-admin-table {
  width: 100%;
  min-width: 720px;
  border-collapse: collapse;
  color: var(--page-ink);
  font-size: 13px;
}

.system-admin-table th,
.system-admin-table td {
  padding: 10px 12px;
  border-right: 2px solid var(--page-line);
  border-bottom: 2px solid var(--page-line);
  text-align: left;
  vertical-align: middle;
  white-space: nowrap;
}

.system-admin-table th {
  background: var(--page-secondary);
  font-size: 12px;
  font-weight: 900;
}

.system-admin-table tr:last-child td {
  border-bottom: 0;
}

.system-admin-table th:last-child,
.system-admin-table td:last-child {
  border-right: 0;
}

.system-admin-chip {
  min-height: 24px;
  display: inline-flex;
  align-items: center;
  padding: 3px 8px;
  border: 2px solid var(--page-line);
  border-radius: 5px;
  background: #ffffff;
  color: var(--page-ink);
  font-size: 12px;
  font-weight: 850;
}

.system-admin-chip--accent {
  background: var(--page-accent);
}

.system-admin-chip--success {
  background: #86efac;
}

.system-admin-chip--warning {
  background: #fda4af;
}

.system-admin-boundaries {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.system-api-key-grid {
  grid-template-columns: minmax(0, 1.1fr) minmax(300px, 0.9fr);
  align-items: stretch;
}

.system-api-key-card,
.system-api-key-guide,
.system-api-key-list {
  display: grid;
  gap: 14px;
}

.system-api-key-form {
  display: grid;
  grid-template-columns: minmax(220px, 1fr) auto;
  gap: 12px;
  align-items: end;
}

.system-api-key-created {
  display: grid;
  gap: 10px;
  padding: 12px;
  border: 2px solid var(--page-line);
  border-radius: 5px;
  background: #dcfce7;
  box-shadow: 4px 4px 0 var(--page-line);
}

.system-api-key-created[hidden] {
  display: none;
}

.system-api-key-secret {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 10px;
}

.system-api-key-secret input,
.system-api-key-form input {
  min-width: 0;
  height: 40px;
  padding: 8px 10px;
  border: 2px solid var(--page-line);
  border-radius: 5px;
  background: #ffffff;
  box-shadow: 4px 4px 0 var(--page-line);
  color: var(--page-ink);
  font: inherit;
  font-size: 13px;
  font-weight: 850;
}

.system-api-key-list .system-admin-table {
  min-width: 760px;
}

.system-api-key-status {
  min-height: 28px;
  color: var(--page-muted);
  font-size: 12px;
  font-weight: 850;
}

.system-api-key-guide pre {
  margin: 0;
  padding: 12px;
  border: 2px solid var(--page-line);
  border-radius: 5px;
  background: #111111;
  color: #f8fafc;
  box-shadow: 4px 4px 0 var(--page-line);
  overflow: auto;
  white-space: pre-wrap;
  font-size: 12px;
  font-weight: 750;
}

.system-admin-boundary-card {
  min-height: 150px;
  gap: 12px;
}

.system-admin-operation-list {
  display: grid;
  gap: 8px;
}

.system-admin-operation-list code,
.system-admin-list-block code {
  display: inline-flex;
  width: fit-content;
  max-width: 100%;
  padding: 5px 7px;
  border: 2px solid var(--page-line);
  border-radius: 5px;
  background: #111111;
  color: #f8fafc;
  font-size: 12px;
  font-weight: 750;
  overflow-wrap: anywhere;
}

.system-admin-context-card {
  position: sticky;
  top: 0;
  display: grid;
  gap: 16px;
  align-self: start;
  background: #ffffff;
}

.system-admin-badge-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.system-admin-list-block {
  display: grid;
  gap: 8px;
  padding-top: 12px;
  border-top: 2px solid var(--page-line);
}

.system-admin-list-block h3 {
  margin: 0;
  color: var(--page-ink);
  font-size: 14px;
  font-weight: 900;
}

.system-admin-list-block span {
  color: var(--page-muted);
  font-size: 13px;
  font-weight: 750;
}

.system-admin-store-block a,
.system-admin-store-chip {
  width: fit-content;
  min-height: 30px;
  padding: 5px 8px;
  background: var(--page-secondary);
  font-size: 12px;
}

.system-sidebar-nav {
  display: grid;
  gap: 4px;
}

.system-sidebar-nav .nav-button__detail {
  width: auto;
  max-width: 92px;
  border-radius: 5px;
}

@media (max-width: 1040px) {
  .system-admin-layout,
  .system-admin-boundaries,
  .system-api-key-grid {
    grid-template-columns: 1fr;
  }

  .system-admin-context-card {
    position: static;
  }
}

@media (max-width: 720px) {
  .system-admin-page {
    padding: 20px 14px;
  }

  .system-admin-hero,
  .system-admin-hero__stats {
    grid-template-columns: 1fr;
  }

  .system-admin-toolbar {
    align-items: stretch;
    flex-direction: column;
  }

  .system-admin-toolbar__actions {
    justify-content: flex-start;
  }

  .system-admin-querybar {
    grid-template-columns: 1fr;
  }

  .system-api-key-form,
  .system-api-key-secret {
    grid-template-columns: 1fr;
  }

  .system-admin-pagination,
  .system-admin-store-link,
  .system-api-key-form .system-admin-command,
  .system-api-key-secret .system-admin-command {
    width: 100%;
    justify-content: center;
  }
}
"#;

const SYSTEM_API_KEY_SCRIPT: &str = r#"
(function(){
  var form = document.getElementById('api-key-create-form');
  var createdPanel = document.getElementById('api-key-created-panel');
  var table = document.getElementById('api-key-table-body');
  var copyButton = document.getElementById('api-key-copy-button');
  var keyInput = document.getElementById('api-key-created-value');
  var status = document.getElementById('api-key-action-status');

  function setStatus(text) {
    if (status) {
      status.textContent = text;
    }
  }

  if (copyButton && keyInput) {
    copyButton.onclick = function() {
      keyInput.select();
      var value = keyInput.value;
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(value).then(function(){
          setStatus('已复制 API Key。');
        }).catch(function(){
          document.execCommand('copy');
          setStatus('已复制 API Key。');
        });
      } else {
        document.execCommand('copy');
        setStatus('已复制 API Key。');
      }
      return false;
    };
  }

  if (form && window.fetch && keyInput) {
    form.addEventListener('submit', function(event) {
      event.preventDefault();
      setStatus('正在创建 API Key...');
      var nameInput = form.querySelector('input[name="name"]');
      var scopeInput = form.querySelector('input[name="scope"]');
      fetch('/api/system/api-key', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'Accept': 'application/json' },
        body: JSON.stringify({
          name: nameInput ? nameInput.value : 'az-aio 调用密钥',
          scope: scopeInput ? scopeInput.value : 'all-services'
        })
      })
      .then(function(response) {
        return response.json().then(function(payload) {
          return { response: response, payload: payload };
        });
      })
      .then(function(result) {
        if (!result.response.ok || !result.payload || !result.payload.data) {
          throw new Error((result.payload && result.payload.msg) || '创建失败');
        }
        keyInput.value = result.payload.data.apiKey || '';
        if (createdPanel) {
          createdPanel.hidden = false;
        }
        setStatus('创建成功，明文密钥只显示一次。');
        loadApiKeys();
      })
      .catch(function(error) {
        setStatus('创建失败：' + (error && error.message ? error.message : error));
      });
    });
  }

  function escapeHtml(value) {
    return String(value || '').replace(/[&<>"']/g, function(ch) {
      return ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[ch]);
    });
  }

  function rowHtml(item) {
    var revoke = item.status === 'active'
      ? '<form method="post" action="/admin-api/system/ui-api-key/revoke" class="system-admin-action-form"><input type="hidden" name="id" value="' + escapeHtml(item.id) + '"><button class="system-admin-command" type="submit">撤销</button></form>'
      : '<span class="system-admin-store-chip">已撤销</span>';
    return '<tr>' +
      '<td>' + escapeHtml(item.name) + '</td>' +
      '<td><code>' + escapeHtml(item.prefix) + '</code></td>' +
      '<td><span class="system-admin-chip system-admin-chip--accent">' + escapeHtml(item.scope) + '</span></td>' +
      '<td><span class="system-admin-chip ' + (item.status === 'active' ? 'system-admin-chip--success' : 'system-admin-chip--warning') + '">' + escapeHtml(item.status) + '</span></td>' +
      '<td>' + escapeHtml(item.createdAt) + '</td>' +
      '<td>' + escapeHtml(item.lastUsedAt || '未使用') + '</td>' +
      '<td>' + revoke + '</td>' +
      '</tr>';
  }

  if (!table || !window.fetch) {
    return;
  }

  function loadApiKeys() {
    fetch('/api/system/api-keys', { headers: { 'Accept': 'application/json' } })
    .then(function(response){ return response.json(); })
    .then(function(payload){
      var items = (payload && payload.data) || [];
      if (!items.length) {
        table.innerHTML = '<tr><td colspan="7">还没有 API 密钥，先在上方创建一把。</td></tr>';
        return;
      }
      table.innerHTML = items.map(rowHtml).join('');
    })
    .catch(function(error){
      table.innerHTML = '<tr><td colspan="7">加载失败：' + escapeHtml(error && error.message ? error.message : error) + '</td></tr>';
    });
  }

  loadApiKeys();
})();
"#;

pub fn SystemAdminPage(context: NativeRenderContext) -> Element {
    let dashboard = system_dashboard_view();
    let route = route_without_query(&context.active_route);
    let page = system_page_for_route(route)
        .map(|page| page.view())
        .or_else(|| system_page_for_route(SYSTEM_DEFAULT_ROUTE).map(|page| page.view()))
        .or_else(|| system_dashboard_view().pages.into_iter().next())
        .unwrap_or_else(empty_system_page);

    rsx! {
        SystemAdminStyle {}
        Page { class: "system-admin-page",
            SystemHero { dashboard: dashboard.clone(), page: page.clone() }
            SystemPageBody {
                page: page.clone(),
                dashboard,
                active_route: context.active_route.clone(),
            }
        }
    }
}

pub fn SystemAdminSidebar(context: NativeRenderContext) -> Element {
    let route = route_without_query(&context.active_route).to_string();
    let sections = system_admin_sections()
        .into_iter()
        .map(SystemSidebarSectionVm::from)
        .collect::<Vec<_>>();

    rsx! {
        SystemAdminStyle {}
        nav { class: "system-sidebar-nav",
            for section in sections {
                div {
                    class: "sidebar-menu-domain",
                    "data-menu-domain": "true",
                    "data-menu-text": section.search_text,
                    p { class: "sidebar__heading", "{section.label}" }
                }
                nav { class: "sidebar-tree sidebar-tree--domain",
                    for node in section.menus {
                        SystemSidebarNode {
                            node,
                            active_route: route.clone(),
                            depth: 0usize,
                        }
                    }
                }
            }
        }
    }
}

fn SystemAdminStyle() -> Element {
    rsx! {
        style {
            "data-az-style": "system-admin-page",
            dangerous_inner_html: SYSTEM_ADMIN_STYLE,
        }
    }
}

#[derive(Clone, PartialEq)]
struct SystemSidebarSectionVm {
    label: String,
    search_text: String,
    menus: Vec<AdminNodeSnapshot>,
}

impl From<crate::system::navigation::AdminSectionSnapshot> for SystemSidebarSectionVm {
    fn from(section: crate::system::navigation::AdminSectionSnapshot) -> Self {
        let search_text = section_search_text(&section);
        Self {
            label: section.label,
            search_text,
            menus: section.menus,
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct SystemSidebarNodeProps {
    node: AdminNodeSnapshot,
    active_route: String,
    depth: usize,
}

fn SystemSidebarNode(props: SystemSidebarNodeProps) -> Element {
    let active = node_is_active(&props.node, &props.active_route);
    let search_text = node_search_text(&props.node);
    let style = tree_style(props.depth);
    let children = props.node.children.clone();

    if !children.is_empty() {
        let detail = format!("{}项", children.len());
        let icon = if props.node.icon.is_empty() {
            "▸".to_string()
        } else {
            props.node.icon.clone()
        };

        return rsx! {
            details {
                class: "sidebar-tree-branch",
                open: active,
                style,
                "data-menu-node": "true",
                "data-menu-text": search_text,
                summary { class: sidebar_branch_class(active),
                    span { class: "nav-button__icon", "{icon}" }
                    span { class: "nav-button__label", "{props.node.label}" }
                    span { class: "nav-button__detail", "{detail}" }
                    span { class: "sidebar-tree-branch__chevron", "⌄" }
                }
                nav { class: "sidebar-tree sidebar-tree--nested",
                    for child in children {
                        SystemSidebarNode {
                            node: child,
                            active_route: props.active_route.clone(),
                            depth: props.depth + 1,
                        }
                    }
                }
            }
        };
    }

    rsx! {
        div {
            class: "sidebar-menu-node",
            style,
            "data-menu-node": "true",
            "data-menu-text": search_text,
            a {
                class: sidebar_leaf_class(active),
                href: "/?route={props.node.href}",
                span { class: "nav-button__icon", "•" }
                span { class: "nav-button__label", "{props.node.label}" }
            }
        }
    }
}

fn sidebar_branch_class(active: bool) -> &'static str {
    if active {
        "nav-button nav-button--plugin nav-button--active nav-button--tree sidebar-tree-branch__summary"
    } else {
        "nav-button nav-button--plugin nav-button--tree sidebar-tree-branch__summary"
    }
}

fn sidebar_leaf_class(active: bool) -> &'static str {
    if active {
        "nav-button nav-button--plugin nav-button--active nav-button--tree"
    } else {
        "nav-button nav-button--plugin nav-button--tree"
    }
}

fn tree_style(depth: usize) -> String {
    let indent = depth * 14;
    let branch_line = indent + 8;
    let parent_line = depth.saturating_sub(1) * 14 + 8;
    format!(
        "--tree-depth: {}; --tree-indent: {}px; --tree-line: {}px; --tree-parent-line: {}px;",
        depth, indent, branch_line, parent_line
    )
}

#[derive(Clone, PartialEq, Props)]
struct SystemHeroProps {
    dashboard: SystemDashboardView,
    page: SystemPageView,
}

fn SystemHero(props: SystemHeroProps) -> Element {
    rsx! {
        Hero { class: "system-admin-hero",
            div {
                Eyebrow { "Bi-Axial Admin / PostgreSQL Contract" }
                h1 { "{props.page.label}" }
                p { "{props.page.description}" }
            }
            div { class: "system-admin-hero__stats",
                StatTile { label: "已接入", value: props.dashboard.implemented_count.to_string() }
                StatTile { label: "参考页", value: props.dashboard.reference_count.to_string() }
                StatTile { label: "PG 表", value: props.dashboard.pg_table_count.to_string() }
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct StatTileProps {
    label: String,
    value: String,
}

fn StatTile(props: StatTileProps) -> Element {
    rsx! {
        div { class: "system-admin-stat",
            span { "{props.label}" }
            strong { "{props.value}" }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct SystemPageBodyProps {
    page: SystemPageView,
    dashboard: SystemDashboardView,
    active_route: String,
}

fn SystemPageBody(props: SystemPageBodyProps) -> Element {
    if props.page.id == "api_keys" {
        return rsx! {
            SystemApiKeyWorkbench {
                page: props.page,
                dashboard: props.dashboard,
                active_route: props.active_route,
            }
        };
    }

    rsx! {
        Split { class: "system-admin-layout",
            div { class: "system-admin-main",
                SystemToolbar { page: props.page.clone() }
                SystemDataTable { page: props.page.clone() }
                SystemBoundaryGrid { page: props.page.clone() }
            }
            SystemContextPanel { page: props.page, dashboard: props.dashboard }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct SystemApiKeyWorkbenchProps {
    page: SystemPageView,
    dashboard: SystemDashboardView,
    active_route: String,
}

fn SystemApiKeyWorkbench(props: SystemApiKeyWorkbenchProps) -> Element {
    let created = query_value(&props.active_route, "created").unwrap_or_default();
    let prefix = query_value(&props.active_route, "prefix").unwrap_or_default();
    let error = query_value(&props.active_route, "error").unwrap_or_default();
    let key_created = created == "1";
    let has_error = !error.is_empty();

    rsx! {
        Split { class: "system-admin-layout",
            div { class: "system-admin-main",
                Card { class: "system-admin-toolbar",
                    div { class: "system-admin-toolbar__title",
                        span { class: "system-admin-mark", "{props.page.icon}" }
                        div {
                            h2 { "API 密钥" }
                            p { "创建后即可用 X-API-Key、Authorization Bearer 或 api_key 查询参数调用所有服务 API。" }
                        }
                    }
                    div { class: "system-admin-toolbar__actions",
                        a {
                            class: "system-admin-command",
                            href: "/api/system/api-keys",
                            "密钥 JSON"
                        }
                        a {
                            class: "system-admin-command",
                            href: "/api/system/status",
                            "系统状态"
                        }
                    }
                }

                Grid { class: "system-api-key-grid",
                    Card { class: "system-api-key-card",
                        BlockTitle {
                            title: "创建新密钥".to_string(),
                            subtitle: "明文 API Key 只显示一次；数据库只保存哈希、前缀、状态和使用时间。".to_string(),
                        }
                        form {
                            id: "api-key-create-form",
                            class: "system-api-key-form",
                            method: "post",
                            action: "/admin-api/system/ui-api-key",
                            label { class: "system-admin-search",
                                span { "密钥名称" }
                                input {
                                    name: "name",
                                    placeholder: "例如：天气服务调用方 / macmini-worker",
                                    value: "az-aio 调用密钥",
                                }
                            }
                            input { r#type: "hidden", name: "scope", value: "all-services" }
                            button {
                                class: "system-admin-command system-admin-command--primary",
                                r#type: "submit",
                                "创建密钥"
                            }
                        }
                        div {
                            id: "api-key-created-panel",
                            class: "system-api-key-created",
                            hidden: !key_created,
                            strong { "新密钥只显示一次，请立即复制保存。" }
                            div { class: "system-api-key-secret",
                                input {
                                    id: "api-key-created-value",
                                    readonly: true,
                                    value: "",
                                    placeholder: "创建成功，密钥前缀：{prefix}",
                                }
                                button {
                                    id: "api-key-copy-button",
                                    class: "system-admin-command",
                                    r#type: "button",
                                    "复制"
                                }
                            }
                        }
                        if has_error {
                            p { class: "system-api-key-status", "错误：{error}" }
                        } else {
                            p { id: "api-key-action-status", class: "system-api-key-status", "准备创建 all-services API Key。" }
                        }
                    }

                    Card { class: "system-api-key-guide",
                        BlockTitle {
                            title: "调用方式".to_string(),
                            subtitle: "同一把 key 默认覆盖当前已暴露服务；显式传入无效 api_key 会返回 401。".to_string(),
                        }
                        pre { "curl -H 'X-API-Key: <api_key>' http://127.0.0.1:18081/api/edge-gateway/assets" }
                        pre { "curl -X POST 'http://127.0.0.1:18081/api/edge-gateway/assets/weather/current?api_key=<api_key>' \\\n  -H 'Content-Type: application/json' \\\n  -d '{{\"latitude\":31.2304,\"longitude\":121.4737,\"timezone\":\"Asia/Shanghai\"}}'" }
                    }
                }

                Card { class: "system-api-key-list",
                    BlockTitle {
                        title: "已创建密钥".to_string(),
                        subtitle: "列表只展示前缀；可在线撤销，撤销后再次使用会被拒绝。".to_string(),
                    }
                    div { class: "system-admin-table-scroll",
                        table { class: "system-admin-table",
                            thead {
                                tr {
                                    th { "名称" }
                                    th { "前缀" }
                                    th { "范围" }
                                    th { "状态" }
                                    th { "创建时间" }
                                    th { "最近使用" }
                                    th { "操作" }
                                }
                            }
                            tbody {
                                id: "api-key-table-body",
                                tr {
                                    td { colspan: "7", "正在加载 API 密钥..." }
                                }
                            }
                        }
                    }
                }
            }
            SystemContextPanel { page: props.page, dashboard: props.dashboard }
        }
        script {
            "data-az-script": "system-api-key-page",
            dangerous_inner_html: SYSTEM_API_KEY_SCRIPT,
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct SystemToolbarProps {
    page: SystemPageView,
}

fn SystemToolbar(props: SystemToolbarProps) -> Element {
    rsx! {
        Card { class: "system-admin-toolbar",
            div { class: "system-admin-toolbar__title",
                span { class: "system-admin-mark", "{props.page.icon}" }
                div {
                    h2 { "{props.page.label}" }
                    p { "{props.page.status_label}" }
                }
            }
            div { class: "system-admin-toolbar__actions",
                for operation in props.page.operations {
                    OperationControl {
                        operation,
                        page_route: props.page.route.clone(),
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct OperationControlProps {
    operation: SystemOperation,
    page_route: String,
}

fn OperationControl(props: OperationControlProps) -> Element {
    rsx! {
        button {
            class: if props.operation.primary {
                "system-admin-command system-admin-command--primary"
            } else {
                "system-admin-command"
            },
            r#type: "button",
            title: format!("{} · {} {}", props.operation.cli, props.operation.method, props.operation.path),
            "data-operation-id": props.operation.id,
            span { "{props.operation.label}" }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct SystemDataTableProps {
    page: SystemPageView,
}

fn SystemDataTable(props: SystemDataTableProps) -> Element {
    rsx! {
        Card { class: "system-admin-table-card",
            BlockTitle {
                title: "业务数据视图".to_string(),
                subtitle: "表格由系统 contract SSR 渲染，正式数据源以 PostgreSQL 表和 store 快照为准。".to_string(),
            }
            div { class: "system-admin-querybar",
                label { class: "system-admin-search",
                    span { "关键词" }
                    input {
                        name: "q",
                        placeholder: "按名称、编码、账号过滤",
                        value: "",
                    }
                }
                div { class: "system-admin-pagination",
                    a { href: format!("/?route={}", props.page.route), "上一页" }
                    span { "1 / 1" }
                    a { href: format!("/?route={}", props.page.route), "下一页" }
                }
                span {
                    class: "system-admin-store-link system-admin-store-link--static",
                    title: format!("/api/system/store/records?page_id={}&o=0&s=20", props.page.id),
                    "PG 数据"
                }
            }
            div { class: "system-admin-table-scroll",
                table { class: "system-admin-table",
                    thead {
                        tr {
                            for column in &props.page.columns {
                                th { style: "width: {column.width};", "{column.label}" }
                            }
                        }
                    }
                    tbody {
                        for row in &props.page.rows {
                            tr {
                                for column in &props.page.columns {
                                    SystemTableCellView {
                                        cell: cell_for_key(row.cells, column.key),
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

#[derive(Clone, PartialEq, Props)]
struct SystemTableCellViewProps {
    cell: SystemTableCell,
}

fn SystemTableCellView(props: SystemTableCellViewProps) -> Element {
    let tone_class = match props.cell.tone {
        Some("accent") => "system-admin-chip system-admin-chip--accent",
        Some("success") => "system-admin-chip system-admin-chip--success",
        Some("warning") => "system-admin-chip system-admin-chip--warning",
        _ => "system-admin-chip",
    };

    rsx! {
        td {
            if props.cell.tone.is_some() {
                span { class: tone_class, "{props.cell.value}" }
            } else {
                span { "{props.cell.value}" }
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct SystemBoundaryGridProps {
    page: SystemPageView,
}

fn SystemBoundaryGrid(props: SystemBoundaryGridProps) -> Element {
    rsx! {
        Grid { class: "system-admin-boundaries",
            Card { class: "system-admin-boundary-card",
                BlockTitle {
                    title: "读取边界".to_string(),
                    subtitle: props.page.read_boundary.clone(),
                }
            }
            Card { class: "system-admin-boundary-card",
                BlockTitle {
                    title: "写入边界".to_string(),
                    subtitle: props.page.write_boundary.clone(),
                }
            }
            Card { class: "system-admin-boundary-card",
                BlockTitle {
                    title: "操作契约".to_string(),
                    subtitle: "REST API 与 CLI 共用同一套 SystemOperation 定义。".to_string(),
                }
                div { class: "system-admin-operation-list",
                    for operation in props.page.operations {
                        code { "{operation.method} {operation.path}" }
                        code { "{operation.cli}" }
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct SystemContextPanelProps {
    page: SystemPageView,
    dashboard: SystemDashboardView,
}

fn SystemContextPanel(props: SystemContextPanelProps) -> Element {
    rsx! {
        Card { class: "system-admin-context-card",
            BlockTitle {
                title: "上下文轴".to_string(),
                subtitle: "顶栏承载 system domain，侧栏承载当前 domain 下的页面树。".to_string(),
            }
            div { class: "system-admin-badge-row",
                Badge { accent: true, "{props.page.status_label}" }
                for permission in props.page.permissions_any_of {
                    Badge { "{permission}" }
                }
            }
            div { class: "system-admin-list-block",
                h3 { "PostgreSQL 表" }
                for table in props.page.pg_tables {
                    code { "{table}" }
                }
            }
            div { class: "system-admin-list-block",
                h3 { "来源模块" }
                for module in props.page.source_modules {
                    span { "{module}" }
                }
            }
            div { class: "system-admin-list-block",
                h3 { "系统总览" }
                span { "Domain: {props.dashboard.domain_id}" }
                span { "Default: {props.dashboard.default_route}" }
            }
            div { class: "system-admin-list-block system-admin-store-block",
                h3 { "后台 store" }
                StoreContractChip { label: "状态接口", path: "/api/system/status".to_string() }
                StoreContractChip { label: "PG 页面快照", path: "/api/system/store/pages".to_string() }
                StoreContractChip {
                    label: "PG 数据快照",
                    path: format!("/api/system/store/records?page_id={}&o=0&s=20", props.page.id),
                }
                StoreContractChip { label: "操作审计", path: "/api/system/store/operations?limit=20".to_string() }
            }
        }
    }
}

#[derive(Clone, PartialEq, Props)]
struct StoreContractChipProps {
    label: &'static str,
    path: String,
}

fn StoreContractChip(props: StoreContractChipProps) -> Element {
    rsx! {
        span {
            class: "system-admin-store-chip",
            title: "{props.path}",
            "{props.label}"
        }
    }
}

fn route_without_query(route: &str) -> &str {
    route.split('?').next().unwrap_or(route)
}

fn query_value(route: &str, key: &str) -> Option<String> {
    let query = route.split_once('?')?.1;
    query.split('&').find_map(|pair| {
        let (raw_key, raw_value) = pair.split_once('=').unwrap_or((pair, ""));
        if raw_key == key {
            urlencoding::decode(raw_value)
                .ok()
                .map(|value| value.into_owned())
        } else {
            None
        }
    })
}

fn section_search_text(section: &crate::system::navigation::AdminSectionSnapshot) -> String {
    format!("{} {}", section.label, section.domain_id)
}

fn node_search_text(node: &AdminNodeSnapshot) -> String {
    format!(
        "{} {} {} {}",
        node.label,
        node.href,
        node.id,
        node.permissions_any_of.join(" ")
    )
}

fn node_is_active(node: &AdminNodeSnapshot, route: &str) -> bool {
    if node.href == route || node.active_patterns.iter().any(|pattern| pattern == route) {
        return true;
    }

    node.children
        .iter()
        .any(|child| node_is_active(child, route))
}

fn cell_for_key(cells: &[SystemTableCell], key: &str) -> SystemTableCell {
    cells
        .iter()
        .copied()
        .find(|cell| cell.key == key)
        .unwrap_or(SystemTableCell {
            key: "",
            value: "",
            tone: None,
        })
}

fn empty_system_page() -> SystemPageView {
    SystemPageView {
        id: "empty".to_string(),
        label: "管理后台".to_string(),
        description: "管理后台页面尚未注册。".to_string(),
        route: SYSTEM_DEFAULT_ROUTE.to_string(),
        icon: "□".to_string(),
        order: 0,
        status: crate::system::catalog::SystemFeatureStatus::ReferenceOnly,
        status_label: "未注册".to_string(),
        source_modules: Vec::new(),
        pg_tables: Vec::new(),
        read_boundary: "暂无读取边界。".to_string(),
        write_boundary: "暂无写入边界。".to_string(),
        permissions_any_of: Vec::new(),
        columns: Vec::new(),
        rows: Vec::new(),
        operations: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use dioxus::prelude::*;

    use super::*;

    #[test]
    fn system_admin_page_renders_pg_boundary() {
        let markup = dioxus_ssr::render_element(rsx! {
            SystemAdminPageFixture {}
        });

        assert!(markup.contains("PostgreSQL 表"));
        assert!(markup.contains("sys_user"));
        assert!(markup.contains("az system user create"));
        assert!(markup.contains("data-operation-id"));
        assert!(markup.contains("/api/system/store/operations"));
        assert!(!markup.contains("href=\"/api/system/users"));
    }

    #[test]
    fn sidebar_marks_active_route() {
        let markup = dioxus_ssr::render_element(rsx! {
            SystemAdminSidebarFixture {}
        });

        assert!(markup.contains("nav-button--active"));
        assert!(markup.contains("sidebar-tree-branch"));
        assert!(markup.contains("<summary"));
        assert!(markup.contains("sidebar-tree--nested"));
        assert!(markup.contains("--tree-indent: 0px"));
        assert!(markup.contains("菜单挂载"));
        assert!(markup.contains("系统配置"));
        assert!(markup.contains("data-menu-text"));
    }

    #[test]
    fn api_key_page_renders_create_and_copy_flow() {
        let markup = dioxus_ssr::render_element(rsx! {
            SystemApiKeyPageFixture {}
        });

        assert!(markup.contains("创建密钥"));
        assert!(markup.contains("api-key-created-value"));
        assert!(markup.contains("/api/system/api-key"));
        assert!(markup.contains("/api/system/api-keys"));
        assert!(markup.contains("X-API-Key"));
    }

    #[component]
    fn SystemAdminPageFixture() -> Element {
        SystemAdminPage(NativeRenderContext {
            active_route: "/system/identity/users".to_string(),
            api_base_url: String::new(),
        })
    }

    #[component]
    fn SystemAdminSidebarFixture() -> Element {
        SystemAdminSidebar(NativeRenderContext {
            active_route: "/system/menu/mounting".to_string(),
            api_base_url: String::new(),
        })
    }

    #[component]
    fn SystemApiKeyPageFixture() -> Element {
        SystemAdminPage(NativeRenderContext {
            active_route: "/system/account/api-keys?key=az_live_test".to_string(),
            api_base_url: String::new(),
        })
    }
}
