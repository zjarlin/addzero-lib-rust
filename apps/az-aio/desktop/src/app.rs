#![forbid(unsafe_code)]

use crate::settings::SettingsPage;
use crate::shell_manager::{ShellManagerPage, ShellManagerRoutePage, ShellPageMode};
use crate::sidebar::{
    SidebarActionButton, SidebarItemModel, SidebarSectionModel, SidebarSectionView,
};
use az_aio_plugin_api::{
    CatalogItemContribution, CatalogItemKind, CatalogSource, CatalogTagContribution,
    CatalogTagGroup, ContributionSet, NavItemContribution, PageContribution, PluginActivation,
    PluginBackendBundle, PluginFrontendBundle, PluginKind, PluginSandboxBackendApiDebug,
    PluginSandboxDebugReport, PluginSandboxUiContributionDebug, PluginState,
    ToolbarActionContribution,
};
use az_aio_plugin_host::{
    HostSnapshot, PluginContributionRecord, PluginRuntimeRecord, load_az_aio_plugin_snapshot,
    set_plugin_enabled,
};
use az_dioxus_components::prelude::{
    AzDataTable, AzDataTableAlign, AzDataTableCell, AzDataTableColumn, AzDataTableRow,
};
use az_git::{GitAccountConfigStore, GitProjectBinding};
use dioxus::prelude::*;
use dioxus::signals::SyncStorage;

const APP_CSS: Asset = asset!("/assets/app.css");
const DEFAULT_ROUTE: &str = "/plugins";
const SETTINGS_ROUTE: &str = "/settings";

const PROJECT_ITEMS: [&str; 7] = [
    "cmp-aio",
    "intellij-aio",
    "aio",
    "addzero-lib-rust",
    "kmp-aio",
    "sub2api",
    "CLIProxyAPI",
];

const RECENT_THREADS: [&str; 3] = [
    "修复 Dioxus 按钮事件透传示例",
    "把新图标接入 AIO 工具栏",
    "将你常用的应用连接到 AZ AIO",
];

struct PluginSnapshotRefresh {
    selected_kind: Option<CatalogItemKind>,
    selected_item_id: Option<Signal<String, SyncStorage>>,
    enabled_items: Signal<Vec<String>, SyncStorage>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SourceFilter {
    All,
    Bundled,
    Community,
    Local,
    System,
    User,
    Wasm,
}

impl SourceFilter {
    fn label(self) -> &'static str {
        match self {
            Self::All => "全部来源",
            Self::Bundled => "预置",
            Self::Community => "社区",
            Self::Local => "本地",
            Self::System => "系统",
            Self::User => "用户",
            Self::Wasm => "Wasm 组件",
        }
    }

    fn matches(self, source: CatalogSource) -> bool {
        match self {
            Self::All => true,
            Self::Bundled => source == CatalogSource::Bundled,
            Self::Community => source == CatalogSource::Community,
            Self::Local => source == CatalogSource::Local,
            Self::System => source == CatalogSource::System,
            Self::User => source == CatalogSource::User,
            Self::Wasm => source == CatalogSource::Wasm,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StatusFilter {
    All,
    Installed,
    Available,
}

impl StatusFilter {
    fn label(self) -> &'static str {
        match self {
            Self::All => "全部",
            Self::Installed => "已启用",
            Self::Available => "可添加",
        }
    }

    fn matches(self, installed: bool) -> bool {
        match self {
            Self::All => true,
            Self::Installed => installed,
            Self::Available => !installed,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PluginMenuView {
    Plugin,
    Skill,
    Cli,
    Env,
}

impl PluginMenuView {
    const ALL: [Self; 4] = [Self::Plugin, Self::Skill, Self::Cli, Self::Env];

    fn label(self) -> &'static str {
        match self {
            Self::Plugin => "插件",
            Self::Skill => "技能",
            Self::Cli => "命令行",
            Self::Env => "环境变量",
        }
    }

    fn catalog_kind(self) -> Option<CatalogItemKind> {
        match self {
            Self::Plugin => Some(CatalogItemKind::Plugin),
            Self::Skill => Some(CatalogItemKind::Skill),
            Self::Cli | Self::Env => None,
        }
    }

    fn shell_mode(self) -> Option<ShellPageMode> {
        match self {
            Self::Cli => Some(ShellPageMode::Cli),
            Self::Env => Some(ShellPageMode::Env),
            Self::Plugin | Self::Skill => None,
        }
    }
}

const SKILL_TAG_ALL_ID: &str = "all";

#[derive(Clone, Copy, PartialEq, Eq)]
struct SkillTagOption {
    id: &'static str,
    label: &'static str,
    group: Option<CatalogTagGroup>,
}

const SKILL_TAG_OPTIONS: [SkillTagOption; 9] = [
    SkillTagOption {
        id: SKILL_TAG_ALL_ID,
        label: "全部",
        group: None,
    },
    SkillTagOption {
        id: "dev.rust",
        label: "Rust",
        group: Some(CatalogTagGroup::Developer),
    },
    SkillTagOption {
        id: "dev.java",
        label: "Java",
        group: Some(CatalogTagGroup::Developer),
    },
    SkillTagOption {
        id: "dev.gradle",
        label: "Gradle",
        group: Some(CatalogTagGroup::Developer),
    },
    SkillTagOption {
        id: "dev.maven",
        label: "Maven",
        group: Some(CatalogTagGroup::Developer),
    },
    SkillTagOption {
        id: "dev.cmp",
        label: "CMP",
        group: Some(CatalogTagGroup::Developer),
    },
    SkillTagOption {
        id: "dev.kmp",
        label: "KMP",
        group: Some(CatalogTagGroup::Developer),
    },
    SkillTagOption {
        id: "dev.convention",
        label: "编程规范",
        group: Some(CatalogTagGroup::Developer),
    },
    SkillTagOption {
        id: "design",
        label: "设计",
        group: Some(CatalogTagGroup::Design),
    },
];

#[allow(non_snake_case)]
#[component]
pub fn App() -> Element {
    let snapshot = use_signal_sync(HostSnapshot::default);
    let snapshot_ready = use_signal_sync(|| false);
    let mut active_route = use_signal(|| DEFAULT_ROUTE.to_string());
    let mut last_app_route = use_signal(|| DEFAULT_ROUTE.to_string());
    let mut sidebar_collapsed = use_signal(|| false);

    use_hook({
        let snapshot = snapshot;
        let snapshot_ready = snapshot_ready;
        move || refresh_plugin_snapshot_async(snapshot, Some(snapshot_ready), None)
    });

    if !*snapshot_ready.read() {
        return rsx! {
            document::Link { rel: "stylesheet", href: APP_CSS }
            PluginShellSkeleton {}
        };
    }

    let snapshot_value = snapshot.read().clone();
    let requested_route = active_route.read().clone();
    let selected_route = requested_route.clone();
    let selected_page = selected_page(&snapshot_value.pages, &selected_route);
    let is_sidebar_collapsed = *sidebar_collapsed.read();
    let shell_class = if is_sidebar_collapsed {
        "az-aio-shell az-aio-shell--collapsed"
    } else {
        "az-aio-shell"
    };
    let body_class = if uses_scroll_body(&selected_page.renderer_id) {
        "workspace__body workspace__body--catalog"
    } else {
        "workspace__body"
    };

    rsx! {
        document::Link { rel: "stylesheet", href: APP_CSS }
        main { class: shell_class,
            TitlebarControls {
                sidebar_collapsed: is_sidebar_collapsed,
                on_toggle_sidebar: move |_| {
                    let collapsed_now = *sidebar_collapsed.read();
                    sidebar_collapsed.set(!collapsed_now);
                },
            }
            AppSidebar {
                nav_items: snapshot_value.nav_items.clone(),
                settings_available: route_available(&snapshot_value, SETTINGS_ROUTE),
                active_route: selected_route.clone(),
                on_route_select: move |route: String| {
                    last_app_route.set(route.clone());
                    active_route.set(route);
                },
            }
            section { class: "workspace",
                HeaderBar {}
                div { class: body_class,
                    match selected_page.renderer_id.as_str() {
                        "catalog" => rsx! { PluginCatalogPage { snapshot } },
                        "git.clis.manager" | "cli-catalog" => rsx! {
                            ShellManagerRoutePage {
                                snapshot,
                                mode: ShellPageMode::Cli,
                            }
                        },
                        "git.envs.manager" | "env-vars" => rsx! {
                            ShellManagerRoutePage {
                                snapshot,
                                mode: ShellPageMode::Env,
                            }
                        },
                        "az-platform-sandbox" => rsx! {
                            AzPlatformSandboxPage { snapshot }
                        },
                        "settings.page" => rsx! {
                            SettingsPage {}
                        },
                        _ => rsx! {
                            EmptyPanel {
                                title: selected_page.title.clone(),
                                mark: selected_page.placeholder_mark.clone(),
                            }
                        },
                    }
                }
            }
        }
    }
}

fn uses_scroll_body(renderer_id: &str) -> bool {
    matches!(renderer_id, "catalog" | "az-platform-sandbox")
}

fn selected_page(pages: &[PageContribution], active_route: &str) -> PageContribution {
    pages
        .iter()
        .find(|page| page.route == active_route)
        .or_else(|| pages.iter().find(|page| page.route == DEFAULT_ROUTE))
        .cloned()
        .unwrap_or_else(|| PageContribution {
            route: DEFAULT_ROUTE.to_string(),
            title: "暂未开放".to_string(),
            subtitle: String::new(),
            renderer_id: "placeholder".to_string(),
            placeholder_mark: "⌘".to_string(),
            order: 0,
        })
}

#[allow(non_snake_case)]
#[component]
fn PluginShellSkeleton() -> Element {
    let primary_rows = [0, 1, 2, 3, 4, 5];
    let project_rows = [0, 1, 2, 3, 4];
    let catalog_rows = [0, 1, 2, 3];

    rsx! {
        main { class: "az-aio-shell plugin-shell-skeleton",
            div { class: "titlebar-controls",
                div { class: "skeleton-icon" }
                div { class: "skeleton-icon skeleton-icon--small" }
                div { class: "skeleton-icon skeleton-icon--small" }
            }
            aside { class: "sidebar skeleton-sidebar",
                div { class: "sidebar__section sidebar__section--primary",
                    nav { class: "sidebar-tree sidebar-tree--primary", aria_label: "加载主导航",
                        for row in primary_rows {
                            div { key: "{row}", class: "skeleton-nav-row",
                                span { class: "skeleton-glyph" }
                                span { class: "skeleton-line skeleton-line--nav" }
                            }
                        }
                    }
                }
                div { class: "sidebar__section",
                    p { class: "sidebar__heading", "项目" }
                    nav { class: "sidebar-tree", aria_label: "加载项目",
                        for row in project_rows {
                            div { key: "{row}", class: "skeleton-nav-row skeleton-nav-row--project",
                                span { class: "skeleton-glyph skeleton-glyph--thin" }
                                span { class: "skeleton-line skeleton-line--project" }
                            }
                        }
                    }
                }
                div { class: "sidebar__footer",
                    div { class: "skeleton-nav-row",
                        span { class: "skeleton-glyph" }
                        span { class: "skeleton-line skeleton-line--nav" }
                    }
                }
            }
            section { class: "workspace",
                HeaderBar {}
                div { class: "workspace__body workspace__body--catalog skeleton-workspace",
                    div { class: "skeleton-catalog",
                        div { class: "skeleton-toolbar",
                            div { class: "skeleton-tabs" }
                            div { class: "skeleton-actions" }
                        }
                        div { class: "skeleton-hero" }
                        div { class: "skeleton-filter-row",
                            div { class: "skeleton-search" }
                            div { class: "skeleton-chip" }
                            div { class: "skeleton-chip" }
                            div { class: "skeleton-chip" }
                        }
                        div { class: "skeleton-content-grid",
                            section { class: "skeleton-list",
                                for row in catalog_rows {
                                    div { key: "{row}", class: "skeleton-card-row",
                                        div { class: "skeleton-card-icon" }
                                        div { class: "skeleton-card-copy",
                                            div { class: "skeleton-line skeleton-line--title" }
                                            div { class: "skeleton-line skeleton-line--body" }
                                        }
                                        div { class: "skeleton-button" }
                                    }
                                }
                            }
                            aside { class: "skeleton-detail",
                                div { class: "skeleton-card-icon skeleton-card-icon--large" }
                                div { class: "skeleton-line skeleton-line--detail-title" }
                                div { class: "skeleton-line skeleton-line--body" }
                                div { class: "skeleton-detail-actions" }
                                div { class: "skeleton-line skeleton-line--body" }
                                div { class: "skeleton-line skeleton-line--body-short" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn TitlebarControls(sidebar_collapsed: bool, on_toggle_sidebar: EventHandler<()>) -> Element {
    let toggle_label = if sidebar_collapsed {
        "展开侧边栏"
    } else {
        "收起侧边栏"
    };

    rsx! {
        div { class: "titlebar-controls",
            button {
                class: "sidebar-toggle",
                r#type: "button",
                aria_label: "{toggle_label}",
                title: "{toggle_label}",
                onclick: move |_| on_toggle_sidebar.call(()),
                span { class: "sidebar-toggle__glyph", "" }
            }
            button { class: "icon-button titlebar-nav", r#type: "button", "‹" }
            button { class: "icon-button titlebar-nav", r#type: "button", "›" }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn AppSidebar(
    nav_items: Vec<NavItemContribution>,
    settings_available: bool,
    active_route: String,
    on_route_select: EventHandler<String>,
) -> Element {
    let primary_items = nav_items
        .iter()
        .map(|item| {
            SidebarItemModel::primary(item.route.clone(), item.label.clone(), item.icon.clone())
        })
        .collect::<Vec<_>>();
    let project_items = sidebar_project_items();
    let recent_items = RECENT_THREADS
        .iter()
        .enumerate()
        .map(|(index, thread)| SidebarItemModel::thread(format!("thread:{index}"), *thread))
        .collect::<Vec<_>>();

    rsx! {
        aside { class: "sidebar",
            SidebarSectionView {
                section: SidebarSectionModel::primary(primary_items),
                active_id: active_route.clone(),
                on_select: move |route: String| on_route_select.call(route),
            }
            SidebarSectionView {
                section: SidebarSectionModel::app_group("项目", project_items),
                active_id: active_route.clone(),
                on_select: move |_route: String| {},
            }
            SidebarSectionView {
                section: SidebarSectionModel::recent("对话", recent_items),
                active_id: active_route.clone(),
                on_select: move |_route: String| {},
            }
            if settings_available {
                div { class: "sidebar__footer",
                    SidebarActionButton {
                        item: SidebarItemModel::settings_action(SETTINGS_ROUTE, "设置"),
                        selected: active_route == SETTINGS_ROUTE,
                        on_select: move |route| on_route_select.call(route),
                    }
                }
            }
        }
    }
}

fn route_available(snapshot: &HostSnapshot, route: &str) -> bool {
    snapshot.pages.iter().any(|page| page.route == route)
}

fn sidebar_project_items() -> Vec<SidebarItemModel> {
    let project_bindings = load_project_bindings();
    if project_bindings.is_empty() {
        return PROJECT_ITEMS
            .iter()
            .map(|project| SidebarItemModel::project(format!("project:{project}"), *project))
            .collect();
    }

    project_bindings
        .into_iter()
        .map(|project| {
            SidebarItemModel::project(format!("project:{}", project.local_path), project.name)
                .with_detail(project.owner)
        })
        .collect()
}

fn load_project_bindings() -> Vec<GitProjectBinding> {
    GitAccountConfigStore::default_store()
        .ok()
        .and_then(|store| store.load().ok())
        .map(|config| config.project_bindings)
        .unwrap_or_default()
}

#[allow(non_snake_case)]
#[component]
fn HeaderBar() -> Element {
    rsx! {
        header { class: "header-bar",
            div { class: "header-bar__actions",
                button { class: "model-button", r#type: "button",
                    span { class: "model-button__mark", "✦" }
                    span { "AZ AIO" }
                    span { class: "model-button__chevron", "⌄" }
                }
                button { class: "icon-button", r#type: "button", "□" }
                button { class: "icon-button", r#type: "button", "◱" }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn PluginCatalogPage(snapshot: Signal<HostSnapshot, SyncStorage>) -> Element {
    let mut active_view = use_signal(|| PluginMenuView::Plugin);
    let mut source_filter = use_signal(|| SourceFilter::All);
    let mut status_filter = use_signal(|| StatusFilter::All);
    let mut skill_tag_filter = use_signal(|| SKILL_TAG_ALL_ID.to_string());
    let mut query = use_signal(String::new);
    let initial_selected_id =
        first_item_id(CatalogItemKind::Plugin, &snapshot.read().catalog_items);
    let mut selected_item_id = use_signal_sync(move || initial_selected_id.clone());
    let enabled_items = use_signal_sync(|| enabled_item_ids(&snapshot.read().catalog_items));

    let host_snapshot = snapshot.read().clone();
    let view = *active_view.read();
    let catalog_kind = view.catalog_kind();
    let source = *source_filter.read();
    let status = *status_filter.read();
    let selected_skill_tag = skill_tag_filter.read().clone();
    let query_text = query.read().trim().to_lowercase();
    let items = host_snapshot.catalog_items.clone();
    let enabled_ids = enabled_items.read().clone();
    let selected_id = selected_item_id.read().clone();
    let selected_kind = catalog_kind.unwrap_or(CatalogItemKind::Plugin);
    let visible_items = catalog_kind.map_or_else(Vec::new, |kind| {
        items
            .iter()
            .filter(|item| {
                item_is_visible(
                    item,
                    kind,
                    source,
                    status,
                    &query_text,
                    &enabled_ids,
                    &selected_skill_tag,
                )
            })
            .cloned()
            .collect()
    });
    let selected_item = selected_catalog_item(selected_kind, &visible_items, &selected_id);
    let effective_selected_id = selected_item.id.clone();
    let selected_item_kind = selected_item.kind;
    let selected_enabled = item_enabled(&enabled_ids, &selected_item.id);
    let catalog_result_count = visible_items.len();
    let visible_enabled_count = visible_items
        .iter()
        .filter(|item| item_enabled(&enabled_ids, &item.id))
        .count();
    let visible_item_ids = visible_items
        .iter()
        .map(|item| item.id.clone())
        .collect::<Vec<_>>();
    let source_options = catalog_kind.map(source_filter_options).unwrap_or_default();
    let sections = catalog_kind
        .map(|kind| catalog_sections(&visible_items, kind))
        .unwrap_or_default();
    let toolbar_actions = if catalog_kind.is_some() {
        route_toolbar_actions(&host_snapshot.toolbar_actions, DEFAULT_ROUTE)
    } else {
        Vec::new()
    };
    let bulk_enable_label = skill_bulk_action_label(&selected_skill_tag, true);
    let bulk_disable_label = skill_bulk_action_label(&selected_skill_tag, false);

    rsx! {
        div { class: "catalog-page",
            div { class: "catalog-toolbar",
                div { class: "segmented segmented--plugin-menu", role: "tablist", aria_label: "插件菜单",
                    for tab in PluginMenuView::ALL {
                        button {
                            class: segmented_class(tab == view),
                            r#type: "button",
                            onclick: move |_| {
                                active_view.set(tab);
                                source_filter.set(SourceFilter::All);
                                skill_tag_filter.set(SKILL_TAG_ALL_ID.to_string());
                                // 命令和环境变量嵌在插件菜单里；只有目录页签需要重置选中的描述符。
                                if let Some(kind) = tab.catalog_kind() {
                                    selected_item_id.set(first_item_id(kind, &snapshot.read().catalog_items));
                                }
                            },
                            "{tab.label()}"
                        }
                    }
                }
                div { class: "catalog-toolbar__actions",
                    for action in toolbar_actions {
                        {
                            let action_id = action.id.clone();
                            let action_class = toolbar_button_class(action.primary);
                            let active_view = view;
                            rsx! {
                                button {
                                    class: action_class,
                                    r#type: "button",
                                    onclick: move |_| {
                                        if action_id == "catalog.refresh" {
                                            refresh_plugin_snapshot_async(
                                                snapshot,
                                                None,
                                                Some(PluginSnapshotRefresh {
                                                    selected_kind: active_view.catalog_kind(),
                                                    selected_item_id: Some(selected_item_id),
                                                    enabled_items,
                                                }),
                                            );
                                        }
                                    },
                                    "{action.icon} {action.label}"
                                }
                            }
                        }
                    }
                }
            }

            section { class: "catalog-hero",
                div { class: "catalog-hero__backdrop" }
                div { class: "catalog-hero__content",
                    h1 { "让 AZ AIO 按你的方式工作" }
                    p { "启用工具、技能和本地工作流，把常用能力固定在桌面端。" }
                }
            }

            if catalog_kind.is_some() {
                div { class: "catalog-controls",
                    label { class: "catalog-search",
                        span { "⌕" }
                        input {
                            value: "{query.read()}",
                            placeholder: "搜索插件或技能",
                            oninput: move |event| query.set(event.value()),
                        }
                    }
                    div { class: "catalog-filters",
                        for option in source_options {
                            button {
                                class: filter_class(option == source),
                                r#type: "button",
                                onclick: move |_| source_filter.set(option),
                                "{option.label()}"
                            }
                        }
                        for option in [StatusFilter::All, StatusFilter::Installed, StatusFilter::Available] {
                            button {
                                class: filter_class(option == status),
                                r#type: "button",
                                onclick: move |_| status_filter.set(option),
                                "{option.label()}"
                            }
                        }
                    }
                }
            }

            if view == PluginMenuView::Skill {
                div { class: "skill-tag-panel",
                    div { class: "skill-tag-panel__row",
                        span { class: "skill-tag-panel__label", "标签" }
                        div { class: "skill-tag-panel__tags",
                            for option in SKILL_TAG_OPTIONS {
                                {
                                    let group_label = option.group.map(CatalogTagGroup::label).unwrap_or("全部技能");
                                    rsx! {
                                        button {
                                            class: skill_tag_filter_class(option.id == selected_skill_tag),
                                            r#type: "button",
                                            title: "{group_label}",
                                            onclick: move |_| skill_tag_filter.set(option.id.to_string()),
                                            "{option.label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "skill-tag-panel__row skill-tag-panel__row--grouped",
                        div { class: "skill-tag-group",
                            span { class: "skill-tag-group__label", "{CatalogTagGroup::Developer.label()}" }
                            p { "Rust / Java / Gradle / Maven / CMP / KMP / 编程规范" }
                        }
                        div { class: "skill-tag-group",
                            span { class: "skill-tag-group__label", "{CatalogTagGroup::Design.label()}" }
                            p { "设计 / UI / UX / 可访问性 / 品牌" }
                        }
                    }
                    div { class: "catalog-bulk-actions",
                        button {
                            class: "catalog-bulk-actions__button",
                            r#type: "button",
                            disabled: visible_item_ids.is_empty(),
                            onclick: {
                                let item_ids = visible_item_ids.clone();
                                move |_| set_items_enabled(enabled_items, item_ids.clone(), true)
                            },
                            "{bulk_enable_label}"
                        }
                        button {
                            class: "catalog-bulk-actions__button catalog-bulk-actions__button--secondary",
                            r#type: "button",
                            disabled: visible_item_ids.is_empty(),
                            onclick: {
                                let item_ids = visible_item_ids.clone();
                                move |_| set_items_enabled(enabled_items, item_ids.clone(), false)
                            },
                            "{bulk_disable_label}"
                        }
                    }
                }
            }

            if catalog_kind.is_some() {
                div { class: "catalog-summary",
                    span { "{view.label()}" }
                    span { "{catalog_result_count} 个结果" }
                    span { "{visible_enabled_count} 已启用" }
                }
            }

            if let Some(kind) = catalog_kind {
                div { class: "catalog-content",
                    div { class: "catalog-list",
                        if catalog_result_count == 0 {
                            div { class: "catalog-empty",
                                div { class: "empty-panel__mark", "⌕" }
                                h2 { "没有匹配项" }
                            }
                        } else {
                            for section in sections {
                                if section_has_visible_items(&visible_items, &section, kind) {
                                    section { class: "catalog-section",
                                        h2 { "{section}" }
                                        div { class: "catalog-section__grid",
                                            for item in visible_items.iter().filter(|item| item.section == section) {
                                                {
                                                    let item_kind = item.kind;
                                                    rsx! {
                                                        CatalogCard {
                                                            item: item.clone(),
                                                            installed: item_enabled(&enabled_ids, &item.id),
                                                            selected: item.id == effective_selected_id,
                                                            on_select: move |id| selected_item_id.set(id),
                                                            on_toggle: move |id| {
                                                                toggle_catalog_item_enabled(
                                                                    snapshot,
                                                                    enabled_items,
                                                                    item_kind,
                                                                    id,
                                                                )
                                                            },
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
                    CatalogDetail {
                        item: selected_item.clone(),
                        installed: selected_enabled,
                        on_toggle: move |id| {
                            toggle_catalog_item_enabled(
                                snapshot,
                                enabled_items,
                                selected_item_kind,
                                id,
                            )
                        },
                    }
                }
            } else if let Some(mode) = view.shell_mode() {
                div { class: "plugin-metadata-content",
                    ShellManagerPage {
                        snapshot,
                        mode,
                        query,
                    }
                }
            }
        }
    }
}

fn route_toolbar_actions(
    actions: &[ToolbarActionContribution],
    route: &str,
) -> Vec<ToolbarActionContribution> {
    actions
        .iter()
        .filter(|action| {
            action
                .route
                .as_deref()
                .is_none_or(|action_route| action_route == route)
        })
        .cloned()
        .collect()
}

fn toolbar_button_class(primary: bool) -> &'static str {
    if primary {
        "toolbar-button toolbar-button--primary"
    } else {
        "toolbar-button"
    }
}

fn segmented_class(selected: bool) -> &'static str {
    if selected {
        "segmented__button segmented__button--active"
    } else {
        "segmented__button"
    }
}

fn filter_class(selected: bool) -> &'static str {
    if selected {
        "filter-chip filter-chip--active"
    } else {
        "filter-chip"
    }
}

fn skill_tag_filter_class(selected: bool) -> &'static str {
    if selected {
        "tag-chip tag-chip--active"
    } else {
        "tag-chip"
    }
}

fn skill_bulk_action_label(selected_tag: &str, enable: bool) -> String {
    let verb = if enable { "启用" } else { "停用" };
    if selected_tag == SKILL_TAG_ALL_ID {
        format!("{verb}全部技能")
    } else {
        format!("{verb}当前标签")
    }
}

fn source_filter_options(kind: CatalogItemKind) -> Vec<SourceFilter> {
    match kind {
        CatalogItemKind::Plugin => vec![
            SourceFilter::All,
            SourceFilter::Bundled,
            SourceFilter::Local,
            SourceFilter::Community,
            SourceFilter::Wasm,
        ],
        CatalogItemKind::Skill => vec![SourceFilter::All, SourceFilter::System, SourceFilter::User],
    }
}

fn catalog_sections(items: &[CatalogItemContribution], kind: CatalogItemKind) -> Vec<String> {
    let mut sections = Vec::new();
    for item in items.iter().filter(|item| item.kind == kind) {
        if !sections.iter().any(|section| section == &item.section) {
            sections.push(item.section.clone());
        }
    }
    sections
}

fn section_has_visible_items(
    items: &[CatalogItemContribution],
    section: &str,
    kind: CatalogItemKind,
) -> bool {
    items
        .iter()
        .any(|item| item.kind == kind && item.section == section)
}

fn first_item_id(kind: CatalogItemKind, items: &[CatalogItemContribution]) -> String {
    items
        .iter()
        .find(|item| item.kind == kind)
        .map(|item| item.id.clone())
        .unwrap_or_default()
}

fn selected_catalog_item(
    kind: CatalogItemKind,
    items: &[CatalogItemContribution],
    selected_id: &str,
) -> CatalogItemContribution {
    items
        .iter()
        .find(|item| item.kind == kind && item.id == selected_id)
        .or_else(|| items.iter().find(|item| item.kind == kind))
        .cloned()
        .unwrap_or_else(|| empty_catalog_item(kind))
}

fn empty_catalog_item(kind: CatalogItemKind) -> CatalogItemContribution {
    CatalogItemContribution {
        id: "catalog.empty".to_string(),
        name: "未发现项目".to_string(),
        description: "当前插件贡献中没有可显示的项目。".to_string(),
        section: "空".to_string(),
        icon: "∅".to_string(),
        accent_class: "plugin-icon--terminal".to_string(),
        kind,
        source: CatalogSource::Local,
        installed: false,
        tags: Vec::new(),
        permissions: Vec::new(),
        path: None,
    }
}

fn enabled_item_ids(items: &[CatalogItemContribution]) -> Vec<String> {
    items
        .iter()
        .filter(|item| item.installed)
        .map(|item| item.id.clone())
        .collect()
}

fn item_enabled(enabled_ids: &[String], item_id: &str) -> bool {
    enabled_ids.iter().any(|id| id == item_id)
}

fn toggle_catalog_item_enabled(
    snapshot: Signal<HostSnapshot, SyncStorage>,
    enabled_items: Signal<Vec<String>, SyncStorage>,
    kind: CatalogItemKind,
    item_id: String,
) {
    let next_enabled = !item_enabled(&enabled_items.read(), &item_id);
    if kind != CatalogItemKind::Plugin {
        set_item_enabled(enabled_items, item_id, next_enabled);
        return;
    }

    if let Err(error) = set_plugin_enabled(&item_id, next_enabled) {
        eprintln!("保存插件启用状态失败：{error}");
        return;
    }

    refresh_plugin_snapshot_async(
        snapshot,
        None,
        Some(PluginSnapshotRefresh {
            selected_kind: Some(kind),
            selected_item_id: None,
            enabled_items,
        }),
    );
}

fn set_item_enabled(
    mut enabled_items: Signal<Vec<String>, SyncStorage>,
    item_id: String,
    enabled: bool,
) {
    let mut items = enabled_items.write();
    if enabled {
        if items.iter().any(|id| id == &item_id) {
            return;
        }
        items.push(item_id);
    } else if let Some(index) = items.iter().position(|id| id == &item_id) {
        items.remove(index);
    }
}

fn set_items_enabled(
    mut enabled_items: Signal<Vec<String>, SyncStorage>,
    item_ids: Vec<String>,
    enabled: bool,
) {
    let mut items = enabled_items.write();
    if enabled {
        for item_id in item_ids {
            if !items.iter().any(|id| id == &item_id) {
                items.push(item_id);
            }
        }
    } else {
        items.retain(|id| !item_ids.iter().any(|item_id| item_id == id));
    }
}

fn refresh_plugin_snapshot_async(
    mut snapshot: Signal<HostSnapshot, SyncStorage>,
    snapshot_ready: Option<Signal<bool, SyncStorage>>,
    refresh: Option<PluginSnapshotRefresh>,
) {
    std::thread::spawn(move || {
        let next_snapshot = load_az_aio_plugin_snapshot();
        if let Some(mut refresh) = refresh {
            if let (Some(kind), Some(mut selected_item_id)) =
                (refresh.selected_kind, refresh.selected_item_id)
            {
                selected_item_id.set(first_item_id(kind, &next_snapshot.catalog_items));
            }
            refresh
                .enabled_items
                .set(enabled_item_ids(&next_snapshot.catalog_items));
        }
        snapshot.set(next_snapshot);
        if let Some(mut snapshot_ready) = snapshot_ready {
            snapshot_ready.set(true);
        }
    });
}

fn item_is_visible(
    item: &CatalogItemContribution,
    kind: CatalogItemKind,
    source: SourceFilter,
    status: StatusFilter,
    query: &str,
    enabled_ids: &[String],
    selected_skill_tag: &str,
) -> bool {
    item.kind == kind
        && source.matches(item.source)
        && status.matches(item_enabled(enabled_ids, &item.id))
        && skill_tag_matches(item, selected_skill_tag)
        && item_matches_query(item, query)
}

fn skill_tag_matches(item: &CatalogItemContribution, selected_skill_tag: &str) -> bool {
    if item.kind != CatalogItemKind::Skill || selected_skill_tag == SKILL_TAG_ALL_ID {
        return true;
    }

    item.tags.iter().any(|tag| tag.id == selected_skill_tag)
}

fn item_matches_query(item: &CatalogItemContribution, query: &str) -> bool {
    if query.trim().is_empty() {
        return true;
    }

    let query = query.to_lowercase();
    item.name.to_lowercase().contains(&query)
        || item.description.to_lowercase().contains(&query)
        || item.section.to_lowercase().contains(&query)
        || item
            .tags
            .iter()
            .any(|tag| tag.label.to_lowercase().contains(&query) || tag.id.contains(&query))
}

fn catalog_tag_class(tag: &CatalogTagContribution) -> &'static str {
    match tag.group {
        CatalogTagGroup::Developer => "catalog-tag catalog-tag--developer",
        CatalogTagGroup::Design => "catalog-tag catalog-tag--design",
    }
}

fn catalog_card_class(selected: bool) -> &'static str {
    if selected {
        "catalog-card catalog-card--selected"
    } else {
        "catalog-card"
    }
}

#[allow(non_snake_case)]
#[component]
fn CatalogCard(
    item: CatalogItemContribution,
    installed: bool,
    selected: bool,
    on_select: EventHandler<String>,
    on_toggle: EventHandler<String>,
) -> Element {
    let icon_class = format!("plugin-icon {}", item.accent_class);
    let card_class = catalog_card_class(selected);
    let action_label = if installed { "停用" } else { "启用" };
    let status_label = if installed { "已启用" } else { "可添加" };
    let action_class = if installed {
        "catalog-card__action catalog-card__action--installed"
    } else {
        "catalog-card__action"
    };
    let select_id = item.id.clone();
    let toggle_id = item.id.clone();

    rsx! {
        article { class: card_class,
            div { class: icon_class, "{item.icon}" }
            div { class: "catalog-card__main",
                div { class: "catalog-card__title-row",
                    h3 { "{item.name}" }
                    span { class: "catalog-card__source", "{item.source.label()}" }
                    span { class: "catalog-card__status", "{status_label}" }
                }
                p { "{item.description}" }
                if !item.tags.is_empty() {
                    div { class: "catalog-card__tags",
                        for tag in item.tags.iter() {
                            span { class: catalog_tag_class(tag), "{tag.label}" }
                        }
                    }
                }
            }
            div { class: "catalog-card__actions",
                button {
                    class: "catalog-card__details",
                    r#type: "button",
                    onclick: move |_| on_select.call(select_id.clone()),
                    "详情"
                }
                button {
                    class: action_class,
                    r#type: "button",
                    onclick: move |_| on_toggle.call(toggle_id.clone()),
                    "{action_label}"
                }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn CatalogDetail(
    item: CatalogItemContribution,
    installed: bool,
    on_toggle: EventHandler<String>,
) -> Element {
    let icon_class = format!("plugin-icon catalog-detail__icon {}", item.accent_class);
    let status_label = if installed { "已启用" } else { "未启用" };
    let action_label = if installed { "停用" } else { "启用" };
    let action_class = if installed {
        "catalog-detail__primary catalog-detail__primary--installed"
    } else {
        "catalog-detail__primary"
    };
    let toggle_id = item.id.clone();

    rsx! {
        aside { class: "catalog-detail",
            div { class: "catalog-detail__header",
                div { class: icon_class, "{item.icon}" }
                div {
                    p { class: "catalog-detail__eyebrow", "{item.kind.label()}" }
                    h2 { "{item.name}" }
                    p { "{item.description}" }
                }
            }
            div { class: "catalog-detail__meta",
                span { "{status_label}" }
                span { "{item.source.label()}" }
                span { "{item.section}" }
            }
            if !item.tags.is_empty() {
                div { class: "catalog-detail__block",
                    h3 { "标签" }
                    div { class: "catalog-detail__tags",
                        for tag in item.tags.iter() {
                            span { class: catalog_tag_class(tag), "{tag.label}" }
                        }
                    }
                }
            }
            div { class: "catalog-detail__actions",
                button {
                    class: action_class,
                    r#type: "button",
                    onclick: move |_| on_toggle.call(toggle_id.clone()),
                    "{action_label}"
                }
                button { class: "catalog-detail__secondary", r#type: "button", "打开配置" }
            }
            div { class: "catalog-detail__block",
                h3 { "权限" }
                div { class: "permission-list",
                    for permission in item.permissions {
                        div { class: "permission-row",
                            span { "✓" }
                            p { "{permission}" }
                        }
                    }
                }
            }
            if let Some(path) = item.path.as_ref() {
                div { class: "catalog-detail__block catalog-detail__path",
                    h3 { "路径" }
                    code { "{path}" }
                }
            }
            div { class: "catalog-detail__block",
                h3 { "运行方式" }
                p { "当前页面渲染插件贡献描述符；所有插件都由运行时 Wasm 包机制装配。" }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn AzPlatformSandboxPage(snapshot: Signal<HostSnapshot, SyncStorage>) -> Element {
    let initial_selected_plugin_id = first_plugin_id(&snapshot.read());
    let mut selected_plugin_id = use_signal(move || initial_selected_plugin_id.clone());

    let host_snapshot = snapshot.read().clone();
    let selected_id = selected_plugin_id.read().clone();
    let plugin_records = host_snapshot.plugins.clone();
    let selected_record = selected_plugin_record(&host_snapshot, &selected_id);
    let selected_contributions = selected_plugin_contributions(&host_snapshot, &selected_id);
    let bundle_plugin_id = selected_record
        .as_ref()
        .map(|record| record.descriptor.id.clone())
        .unwrap_or_else(|| selected_id.clone());
    let frontend_bundle = frontend_bundle_contract(&bundle_plugin_id, &selected_contributions);
    let backend_bundle = backend_bundle_contract(&bundle_plugin_id, &selected_contributions);
    let sandbox_debug = PluginSandboxDebugReport::from_contributions(&selected_contributions);
    let ui_rows = ui_contribution_rows(&sandbox_debug.ui_contributions);
    let backend_rows = backend_api_rows(&sandbox_debug.backend_apis);
    let plugin_count = plugin_records.len();
    let ui_count = host_snapshot.ui_contributions.len();
    let backend_count = host_snapshot.backend_apis.len();

    rsx! {
        div { class: "az-platform-page",
            header { class: "metadata-header az-platform-header",
                div { class: "metadata-header__mark", "◇" }
                div {
                    h1 { "az-platform" }
                    p { "{plugin_count} 个插件 / {ui_count} 个 UI 贡献 / {backend_count} 个后端接口" }
                }
            }

            div { class: "az-platform-workbench",
                aside { class: "az-platform-plugin-list",
                    for plugin in plugin_records {
                        {
                            let plugin_id = plugin.descriptor.id.clone();
                            let display_id = plugin_id.clone();
                            let plugin_name = plugin.descriptor.name.clone();
                            let button_class =
                                sandbox_plugin_button_class(display_id == selected_id, &plugin.state);
                            let state_label = plugin_state_label(&plugin.state);
                            rsx! {
                                button {
                                    class: button_class,
                                    r#type: "button",
                                    onclick: move |_| selected_plugin_id.set(plugin_id.clone()),
                                    strong { "{plugin_name}" }
                                    code { "{display_id}" }
                                    span { "{state_label}" }
                                }
                            }
                        }
                    }
                }

                section { class: "az-platform-detail",
                    if let Some(record) = selected_record.as_ref() {
                        SandboxPluginSummary {
                            record: record.clone(),
                            contributions: selected_contributions.clone(),
                        }
                        SandboxBundleContract {
                            frontend: frontend_bundle.clone(),
                            backend: backend_bundle.clone(),
                        }
                        section { class: "az-platform-section",
                            div { class: "az-platform-section__header",
                                h2 { "UI 贡献位" }
                                span { "{selected_contributions.ui_contributions.len()}" }
                            }
                            AzDataTable {
                                columns: ui_contribution_columns(),
                                rows: ui_rows,
                                empty_label: "没有 UI 贡献".to_string(),
                                dense: true,
                                bordered: true,
                            }
                        }
                        section { class: "az-platform-section",
                            div { class: "az-platform-section__header",
                                h2 { "后端接口" }
                                span { "{selected_contributions.backend_apis.len()}" }
                            }
                            AzDataTable {
                                columns: backend_api_columns(),
                                rows: backend_rows,
                                empty_label: "没有后端接口".to_string(),
                                dense: true,
                                bordered: true,
                            }
                        }
                    } else {
                        div { class: "catalog-empty",
                            div { class: "empty-panel__mark", "◇" }
                            h2 { "未选择插件" }
                        }
                    }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn SandboxBundleContract(frontend: PluginFrontendBundle, backend: PluginBackendBundle) -> Element {
    let frontend_nav_count = frontend.nav_items.len();
    let frontend_page_count = frontend.pages.len();
    let frontend_ui_count = frontend.ui_contributions.len();
    let frontend_action_count = frontend.toolbar_actions.len();
    let frontend_catalog_count = frontend.catalog_providers.len();
    let frontend_settings_count = frontend.settings_sections.len();
    let frontend_settings_default_count = frontend
        .settings_sections
        .iter()
        .map(|section| section.defaults.len())
        .sum::<usize>();
    let settings_defaults = settings_default_rows(&frontend);
    let backend_api_count = backend.backend_apis.len();
    let backend_shell_count = backend.shell_entries.len();
    let backend_generated_count = backend.generated_files.len();

    rsx! {
        section { class: "az-platform-section",
            div { class: "az-platform-section__header",
                h2 { "包契约" }
                span { "2" }
            }
            div { class: "az-platform-bundle-grid",
                div { class: "az-platform-bundle",
                    div { class: "az-platform-bundle__title",
                        strong { "Frontend" }
                        code { "frontend/az-frontend.json" }
                    }
                    div { class: "az-platform-bundle__metrics",
                        BundleMetric { label: "导航", value: frontend_nav_count }
                        BundleMetric { label: "页面", value: frontend_page_count }
                        BundleMetric { label: "UI", value: frontend_ui_count }
                        BundleMetric { label: "动作", value: frontend_action_count }
                        BundleMetric { label: "目录", value: frontend_catalog_count }
                        BundleMetric { label: "设置", value: frontend_settings_count }
                        BundleMetric { label: "默认值", value: frontend_settings_default_count }
                    }
                    if !settings_defaults.is_empty() {
                        div { class: "az-platform-bundle-defaults",
                            for default in settings_defaults {
                                div { class: "az-platform-bundle-default",
                                    span { class: "az-platform-bundle-default__section", "{default.section_label}" }
                                    strong { "{default.label}" }
                                    code { "{default.key}" }
                                    span { class: "az-platform-bundle-default__value", "{default.value}" }
                                    p { "{default.description}" }
                                }
                            }
                        }
                    }
                }
                div { class: "az-platform-bundle",
                    div { class: "az-platform-bundle__title",
                        strong { "Backend" }
                        code { "backend/az-backend.json" }
                    }
                    div { class: "az-platform-bundle__metrics az-platform-bundle__metrics--backend",
                        BundleMetric { label: "API", value: backend_api_count }
                        BundleMetric { label: "Shell", value: backend_shell_count }
                        BundleMetric { label: "生成文件", value: backend_generated_count }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SandboxSettingsDefaultRow {
    section_label: String,
    key: String,
    label: String,
    value: String,
    description: String,
}

fn settings_default_rows(frontend: &PluginFrontendBundle) -> Vec<SandboxSettingsDefaultRow> {
    frontend
        .settings_sections
        .iter()
        .flat_map(|section| {
            section
                .defaults
                .iter()
                .map(|default| SandboxSettingsDefaultRow {
                    section_label: section.label.clone(),
                    key: default.key.clone(),
                    label: default.label.clone(),
                    value: default.value.clone(),
                    description: default.description.clone(),
                })
        })
        .collect()
}

#[allow(non_snake_case)]
#[component]
fn BundleMetric(label: &'static str, value: usize) -> Element {
    rsx! {
        div { class: "az-platform-bundle__metric",
            span { "{label}" }
            strong { "{value}" }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn SandboxPluginSummary(record: PluginRuntimeRecord, contributions: ContributionSet) -> Element {
    let descriptor = record.descriptor;
    let state_label = plugin_state_label(&record.state);
    let state_class = plugin_state_pill_class(&record.state);
    let activation_label = plugin_activation_label(&descriptor.activation);
    let kind_label = plugin_kind_label(&descriptor.kind);
    let ui_count = contributions.ui_contributions.len();
    let backend_count = contributions.backend_apis.len();
    let nav_count = contributions.nav_items.len();
    let page_count = contributions.pages.len();
    let capabilities = descriptor.capabilities.clone();
    let permissions = descriptor.permissions.clone();

    rsx! {
        section { class: "az-platform-summary",
            div { class: "az-platform-summary__title",
                div {
                    p { class: "metadata-summary__eyebrow", "{descriptor.id}" }
                    h2 { "{descriptor.name}" }
                    p { "{descriptor.description}" }
                }
                span { class: state_class, "{state_label}" }
            }
            div { class: "az-platform-summary__meta",
                span { "{kind_label}" }
                span { "{activation_label}" }
                span { "v{descriptor.version}" }
                span { "priority {descriptor.priority}" }
            }
            div { class: "az-platform-stat-grid",
                SandboxStat { label: "页面", value: page_count }
                SandboxStat { label: "导航", value: nav_count }
                SandboxStat { label: "UI", value: ui_count }
                SandboxStat { label: "API", value: backend_count }
            }
            div { class: "az-platform-chip-block",
                h3 { "能力" }
                div { class: "az-platform-chip-list",
                    if capabilities.is_empty() {
                        span { class: "az-platform-muted", "无" }
                    } else {
                        for capability in capabilities {
                            span { class: "az-platform-chip", "{capability}" }
                        }
                    }
                }
            }
            div { class: "az-platform-chip-block",
                h3 { "权限" }
                div { class: "az-platform-chip-list",
                    if permissions.is_empty() {
                        span { class: "az-platform-muted", "无" }
                    } else {
                        for permission in permissions {
                            span { class: "az-platform-chip az-platform-chip--permission", "{permission}" }
                        }
                    }
                }
            }
            if let Some(error) = record.error.as_ref() {
                div { class: "settings-message settings-message--error", "{error}" }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn SandboxStat(label: &'static str, value: usize) -> Element {
    rsx! {
        div { class: "az-platform-stat",
            span { "{label}" }
            strong { "{value}" }
        }
    }
}

fn first_plugin_id(snapshot: &HostSnapshot) -> String {
    snapshot
        .plugins
        .first()
        .map(|plugin| plugin.descriptor.id.clone())
        .unwrap_or_default()
}

fn selected_plugin_record(snapshot: &HostSnapshot, plugin_id: &str) -> Option<PluginRuntimeRecord> {
    snapshot
        .plugins
        .iter()
        .find(|plugin| plugin.descriptor.id == plugin_id)
        .or_else(|| snapshot.plugins.first())
        .cloned()
}

fn selected_plugin_contributions(snapshot: &HostSnapshot, plugin_id: &str) -> ContributionSet {
    selected_plugin_contribution_record(snapshot, plugin_id)
        .map(|record| record.contributions)
        .unwrap_or_default()
}

fn selected_plugin_contribution_record(
    snapshot: &HostSnapshot,
    plugin_id: &str,
) -> Option<PluginContributionRecord> {
    snapshot
        .plugin_contributions
        .iter()
        .find(|record| record.plugin_id == plugin_id)
        .or_else(|| snapshot.plugin_contributions.first())
        .cloned()
}

fn frontend_bundle_contract(
    plugin_id: &str,
    contributions: &ContributionSet,
) -> PluginFrontendBundle {
    PluginFrontendBundle {
        schema_version: PluginFrontendBundle::SCHEMA_VERSION,
        plugin_id: plugin_id.to_string(),
        nav_items: contributions.nav_items.clone(),
        pages: contributions.pages.clone(),
        ui_contributions: contributions.ui_contributions.clone(),
        toolbar_actions: contributions.toolbar_actions.clone(),
        catalog_providers: contributions.catalog_providers.clone(),
        settings_sections: contributions.settings_sections.clone(),
    }
}

fn backend_bundle_contract(
    plugin_id: &str,
    contributions: &ContributionSet,
) -> PluginBackendBundle {
    PluginBackendBundle {
        schema_version: PluginBackendBundle::SCHEMA_VERSION,
        plugin_id: plugin_id.to_string(),
        backend_apis: contributions.backend_apis.clone(),
        shell_entries: contributions.shell_entries.clone(),
        generated_files: contributions.generated_files.clone(),
    }
}

fn sandbox_plugin_button_class(selected: bool, state: &PluginState) -> &'static str {
    match (selected, state) {
        (true, PluginState::Failed) => {
            "az-platform-plugin az-platform-plugin--selected az-platform-plugin--failed"
        }
        (true, _) => "az-platform-plugin az-platform-plugin--selected",
        (false, PluginState::Failed) => "az-platform-plugin az-platform-plugin--failed",
        (false, _) => "az-platform-plugin",
    }
}

fn plugin_state_pill_class(state: &PluginState) -> &'static str {
    match state {
        PluginState::Failed => "metadata-status metadata-status--failed",
        PluginState::Active | PluginState::Loaded => "metadata-status metadata-status--generated",
        PluginState::Discovered | PluginState::Disabled => "metadata-status",
    }
}

fn plugin_state_label(state: &PluginState) -> &'static str {
    match state {
        PluginState::Discovered => "已发现",
        PluginState::Loaded => "已加载",
        PluginState::Active => "已启用",
        PluginState::Disabled => "已禁用",
        PluginState::Failed => "失败",
    }
}

fn plugin_activation_label(activation: &PluginActivation) -> &'static str {
    match activation {
        PluginActivation::Eager => "立即启用",
        PluginActivation::Lazy => "延迟启用",
    }
}

fn plugin_kind_label(kind: &PluginKind) -> &'static str {
    match kind {
        PluginKind::WasmComponent => "wasm-component",
        PluginKind::Native => "native",
    }
}

fn table_column(key: &str, header: &str) -> AzDataTableColumn {
    AzDataTableColumn {
        key: key.to_string(),
        header: header.to_string(),
        class: None,
        align: AzDataTableAlign::Start,
    }
}

fn ui_contribution_columns() -> Vec<AzDataTableColumn> {
    vec![
        table_column("slot", "插槽"),
        table_column("label", "名称"),
        table_column("renderer", "渲染器"),
        table_column("route", "路由"),
    ]
}

fn backend_api_columns() -> Vec<AzDataTableColumn> {
    vec![
        table_column("request", "请求"),
        table_column("label", "名称"),
        table_column("description", "说明"),
    ]
}

fn ui_contribution_rows(items: &[PluginSandboxUiContributionDebug]) -> Vec<AzDataTableRow> {
    items
        .iter()
        .map(|item| AzDataTableRow {
            key: item.id.clone(),
            cells: vec![
                item.slot_label.clone().into(),
                item.label.clone().into(),
                item.renderer_id.clone().into(),
                route_cell(item.route.as_deref()),
            ],
            class: None,
        })
        .collect()
}

fn backend_api_rows(items: &[PluginSandboxBackendApiDebug]) -> Vec<AzDataTableRow> {
    items
        .iter()
        .map(|item| AzDataTableRow {
            key: item.id.clone(),
            cells: vec![
                item.request_hint.clone().into(),
                item.label.clone().into(),
                item.description.clone().into(),
            ],
            class: None,
        })
        .collect()
}

fn route_cell(route: Option<&str>) -> AzDataTableCell {
    route.unwrap_or("全局").into()
}

#[allow(non_snake_case)]
#[component]
fn EmptyPanel(title: String, mark: String) -> Element {
    rsx! {
        div { class: "empty-panel",
            div { class: "empty-panel__mark", "{mark}" }
            h1 { "{title}" }
        }
    }
}
