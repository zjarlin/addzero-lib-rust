#![forbid(unsafe_code)]

use crate::settings::SettingsPage;
use crate::shell_manager::{ShellManagerPage, ShellManagerRoutePage, ShellPageMode};
use crate::sidebar::{
    SidebarActionButton, SidebarItemModel, SidebarSectionModel, SidebarSectionView,
};
use codex_plugin_api::{
    CatalogItemContribution, CatalogItemKind, CatalogSource, CatalogTagContribution,
    CatalogTagGroup, PageContribution, PageRenderer, ToolbarActionContribution,
};
use codex_plugin_host::{HostSnapshot, load_codex_plugin_snapshot};
use dioxus::prelude::*;

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
    "Finish the Dioxus button pass-through example",
    "Wire the new icon into the AIO toolbar",
    "将你常用的应用连接到 Codex",
];

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
            Self::Wasm => "Wasm",
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
    let snapshot = use_signal(load_codex_plugin_snapshot);
    let mut active_route = use_signal(|| DEFAULT_ROUTE.to_string());
    let mut last_app_route = use_signal(|| DEFAULT_ROUTE.to_string());
    let mut sidebar_collapsed = use_signal(|| false);

    let snapshot_value = snapshot.read().clone();
    let selected_route = active_route.read().clone();
    let selected_page = selected_page(&snapshot_value.pages, &selected_route);
    let is_sidebar_collapsed = *sidebar_collapsed.read();
    let shell_class = if is_sidebar_collapsed {
        "codex-shell codex-shell--collapsed"
    } else {
        "codex-shell"
    };
    let body_class = if uses_scroll_body(selected_page.renderer) {
        "workspace__body workspace__body--catalog"
    } else {
        "workspace__body"
    };

    if selected_route == SETTINGS_ROUTE {
        let return_route = last_app_route.read().clone();
        return rsx! {
            document::Link { rel: "stylesheet", href: APP_CSS }
            SettingsPage {
                on_return: move |_| active_route.set(return_route.clone()),
            }
        };
    }

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
                snapshot: snapshot_value,
                active_route: selected_route.clone(),
                on_route_select: move |route: String| {
                    last_app_route.set(route.clone());
                    active_route.set(route);
                },
                on_settings_select: move |_| active_route.set(SETTINGS_ROUTE.to_string()),
            }
            section { class: "workspace",
                HeaderBar {}
                div { class: body_class,
                    match selected_page.renderer {
                        PageRenderer::Catalog => rsx! { PluginCatalogPage { snapshot } },
                        PageRenderer::CliCatalog => rsx! {
                            ShellManagerRoutePage {
                                snapshot,
                                mode: ShellPageMode::Cli,
                            }
                        },
                        PageRenderer::EnvVars => rsx! {
                            ShellManagerRoutePage {
                                snapshot,
                                mode: ShellPageMode::Env,
                            }
                        },
                        PageRenderer::Placeholder => rsx! {
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

fn uses_scroll_body(renderer: PageRenderer) -> bool {
    matches!(renderer, PageRenderer::Catalog)
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
            renderer: PageRenderer::Placeholder,
            placeholder_mark: "⌘".to_string(),
            order: 0,
        })
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
    snapshot: HostSnapshot,
    active_route: String,
    on_route_select: EventHandler<String>,
    on_settings_select: EventHandler<()>,
) -> Element {
    let primary_items = snapshot
        .nav_items
        .iter()
        .map(|item| {
            SidebarItemModel::primary(item.route.clone(), item.label.clone(), item.icon.clone())
        })
        .collect::<Vec<_>>();
    let project_items = PROJECT_ITEMS
        .iter()
        .map(|project| SidebarItemModel::project(format!("project:{project}"), *project))
        .collect::<Vec<_>>();
    let recent_items = RECENT_THREADS
        .iter()
        .enumerate()
        .map(|(index, thread)| SidebarItemModel::thread(format!("thread:{index}"), *thread))
        .collect::<Vec<_>>();
    let settings_item = SidebarItemModel::settings_action(SETTINGS_ROUTE, "设置");

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
            div { class: "sidebar__footer",
                SidebarActionButton {
                    item: settings_item,
                    selected: false,
                    on_select: move |_| on_settings_select.call(()),
                }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn HeaderBar() -> Element {
    rsx! {
        header { class: "header-bar",
            div { class: "header-bar__actions",
                button { class: "model-button", r#type: "button",
                    span { class: "model-button__mark", "✦" }
                    span { "Codex" }
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
fn PluginCatalogPage(snapshot: Signal<HostSnapshot>) -> Element {
    let mut active_view = use_signal(|| PluginMenuView::Plugin);
    let mut source_filter = use_signal(|| SourceFilter::All);
    let mut status_filter = use_signal(|| StatusFilter::All);
    let mut skill_tag_filter = use_signal(|| SKILL_TAG_ALL_ID.to_string());
    let mut query = use_signal(String::new);
    let initial_selected_id =
        first_item_id(CatalogItemKind::Plugin, &snapshot.read().catalog_items);
    let mut selected_item_id = use_signal(move || initial_selected_id.clone());
    let mut enabled_items = use_signal(|| enabled_item_ids(&snapshot.read().catalog_items));

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
                div { class: "segmented segmented--plugin-menu", role: "tablist", aria_label: "Plugin menu",
                    for tab in PluginMenuView::ALL {
                        button {
                            class: segmented_class(tab == view),
                            r#type: "button",
                            onclick: move |_| {
                                active_view.set(tab);
                                source_filter.set(SourceFilter::All);
                                skill_tag_filter.set(SKILL_TAG_ALL_ID.to_string());
                                // Shell metadata is intentionally nested under the plugin menu, so only
                                // catalog-backed tabs need to reset the selected catalog descriptor.
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
                                            let next_snapshot = load_codex_plugin_snapshot();
                                            if let Some(kind) = active_view.catalog_kind() {
                                                selected_item_id.set(first_item_id(kind, &next_snapshot.catalog_items));
                                            }
                                            enabled_items.set(enabled_item_ids(&next_snapshot.catalog_items));
                                            snapshot.set(next_snapshot);
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
                    h1 { "让 Codex 按你的方式工作" }
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
                                                CatalogCard {
                                                    item: item.clone(),
                                                    installed: item_enabled(&enabled_ids, &item.id),
                                                    selected: item.id == effective_selected_id,
                                                    on_select: move |id| selected_item_id.set(id),
                                                    on_toggle: move |id| toggle_enabled(enabled_items, id),
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
                        on_toggle: move |id| toggle_enabled(enabled_items, id),
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

fn toggle_enabled(mut enabled_items: Signal<Vec<String>>, item_id: String) {
    let mut items = enabled_items.write();
    if let Some(index) = items.iter().position(|id| id == &item_id) {
        items.remove(index);
    } else {
        items.push(item_id);
    }
}

fn set_items_enabled(mut enabled_items: Signal<Vec<String>>, item_ids: Vec<String>, enabled: bool) {
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
                p { "当前页面只渲染插件贡献描述符；native builtin 和外部 WIT 组件由 host 统一装配。" }
            }
        }
    }
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
