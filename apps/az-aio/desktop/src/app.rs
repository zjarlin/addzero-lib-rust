#![forbid(unsafe_code)]

use crate::sidebar::{SidebarItemModel, SidebarSectionModel, SidebarSectionView};
use az_aio_plugin_api::api::{
    NativeRenderContext, NavItemContribution, PageContribution, UiContribution, UiContributionSlot,
};
use az_aio_plugin_host::host::{
    native_renderer, start_native_loopback_server, HostSnapshot,
};
use dioxus::prelude::*;
use dioxus::signals::SyncStorage;

const APP_CSS: &str = include_str!("../assets/app.css");
const DEFAULT_ROUTE: &str = "/assets";
const SETTINGS_ROUTE: &str = "/settings";

#[allow(non_snake_case)]
#[component]
pub fn App() -> Element {
    let snapshot = use_signal_sync(HostSnapshot::default);
    let snapshot_ready = use_signal_sync(|| false);
    let api_base_url = use_signal_sync(String::new);
    let mut active_route = use_signal(|| DEFAULT_ROUTE.to_string());
    let mut sidebar_collapsed = use_signal(|| false);

    use_hook({
        let snapshot = snapshot;
        let snapshot_ready = snapshot_ready;
        let api_base_url = api_base_url;
        move || load_native_snapshot_async(snapshot, snapshot_ready, api_base_url)
    });

    if !*snapshot_ready.read() {
        return rsx! {
            document::Style { "{APP_CSS}" }
            ShellSkeleton {}
        };
    }

    let snapshot_value = snapshot.read().clone();
    let selected_route = normalize_active_route(&snapshot_value, &active_route.read());
    if selected_route != *active_route.read() {
        active_route.set(selected_route.clone());
    }
    let selected_page = selected_page(&snapshot_value.pages, &selected_route);
    let shell_class = if *sidebar_collapsed.read() {
        "az-aio-shell az-aio-shell--collapsed"
    } else {
        "az-aio-shell"
    };
    let api_base_url_value = api_base_url.read().clone();

    rsx! {
        document::Style { "{APP_CSS}" }
        main { class: shell_class,
            TitlebarControls {
                sidebar_collapsed: *sidebar_collapsed.read(),
                on_toggle_sidebar: move |_| {
                    let collapsed = *sidebar_collapsed.read();
                    sidebar_collapsed.set(!collapsed);
                },
            }
            ShellSidebar {
                nav_items: snapshot_value.nav_items.clone(),
                settings_available: settings_available(&snapshot_value),
                active_route: selected_route.clone(),
                on_route_select: move |route: String| active_route.set(route),
            }
            section { class: "workspace",
                {render_named_slot(
                    &snapshot_value,
                    UiContributionSlot::AppTopbar,
                    &selected_route,
                    &api_base_url_value,
                    HeaderBar(),
                )}
                div { class: "workspace__body",
                    {render_route_content(
                        &snapshot_value,
                        &selected_page,
                        &selected_route,
                        &api_base_url_value,
                    )}
                }
            }
            {render_optional_slot_panel(
                &snapshot_value,
                UiContributionSlot::SandboxPanel,
                &selected_route,
                &api_base_url_value,
                "workspace__right-panel",
            )}
        }
    }
}

fn load_native_snapshot_async(
    mut snapshot: Signal<HostSnapshot, SyncStorage>,
    mut snapshot_ready: Signal<bool, SyncStorage>,
    mut api_base_url: Signal<String, SyncStorage>,
) {
    std::thread::spawn(move || {
        let context = az_aio_plugin_api::api::NativePluginContext {
            api_base_url: "http://127.0.0.1:0".to_string(),
            database_url: None,
            config_dir: std::path::PathBuf::from("."),
            data_dir: std::path::PathBuf::from("."),
        };
        az_aio_plugin_bundled::api::ensure_linked();
        let next_snapshot =
            az_aio_plugin_host::host::load_az_aio_native_snapshot(context);
        let loopback_url = start_loopback_server(next_snapshot.clone()).unwrap_or_default();
        snapshot.set(next_snapshot);
        api_base_url.set(loopback_url);
        snapshot_ready.set(true);
    });
}

fn start_loopback_server(snapshot: HostSnapshot) -> Option<String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .ok()?;
    runtime
        .block_on(start_native_loopback_server(snapshot))
        .ok()
}

fn normalize_active_route(snapshot: &HostSnapshot, active_route: &str) -> String {
    if route_available(snapshot, active_route) {
        return active_route.to_string();
    }
    snapshot
        .pages
        .first()
        .map(|page| page.route.clone())
        .unwrap_or_else(|| DEFAULT_ROUTE.to_string())
}

fn selected_page(pages: &[PageContribution], active_route: &str) -> PageContribution {
    pages
        .iter()
        .find(|page| page.route == active_route)
        .cloned()
        .unwrap_or_else(|| PageContribution {
            route: active_route.to_string(),
            title: "No Plugin Renderer".to_string(),
            subtitle: String::new(),
            renderer_id: "placeholder".to_string(),
            placeholder_mark: "?".to_string(),
            order: 0,
        })
}

fn route_available(snapshot: &HostSnapshot, route: &str) -> bool {
    snapshot.pages.iter().any(|page| page.route == route)
        || snapshot
            .ui_contributions
            .iter()
            .any(|contribution| contribution.route.as_deref() == Some(route))
}

fn settings_available(snapshot: &HostSnapshot) -> bool {
    snapshot
        .ui_contributions
        .iter()
        .any(|contribution| contribution.slot == UiContributionSlot::SettingsContent)
}

fn render_route_content(
    snapshot: &HostSnapshot,
    selected_page: &PageContribution,
    active_route: &str,
    api_base_url: &str,
) -> Element {
    if active_route == SETTINGS_ROUTE {
        return render_slot_or_fallback(
            snapshot,
            UiContributionSlot::SettingsContent,
            active_route,
            api_base_url,
            EmptyPanel("Settings".to_string(), "#".to_string()),
        );
    }

    if active_route.starts_with("/project") {
        return rsx! {
            div { class: "project-workspace",
                {render_optional_slot_panel(
                    snapshot,
                    UiContributionSlot::ProjectSidebar,
                    active_route,
                    api_base_url,
                    "project-workspace__sidebar",
                )}
                div { class: "project-workspace__content",
                    {render_slot_or_fallback(
                        snapshot,
                        UiContributionSlot::ProjectContent,
                        active_route,
                        api_base_url,
                        EmptyPanel(
                            selected_page.title.clone(),
                            selected_page.placeholder_mark.clone(),
                        ),
                    )}
                }
            }
        };
    }

    if let Some(render) = native_renderer(snapshot, &selected_page.renderer_id) {
        return render(NativeRenderContext {
            active_route: active_route.to_string(),
            api_base_url: api_base_url.to_string(),
        });
    }

    render_slot_or_fallback(
        snapshot,
        UiContributionSlot::Content,
        active_route,
        api_base_url,
        EmptyPanel(
            selected_page.title.clone(),
            selected_page.placeholder_mark.clone(),
        ),
    )
}

fn render_named_slot(
    snapshot: &HostSnapshot,
    slot: UiContributionSlot,
    active_route: &str,
    api_base_url: &str,
    fallback: Element,
) -> Element {
    render_slot_or_fallback(snapshot, slot, active_route, api_base_url, fallback)
}

fn render_slot_or_fallback(
    snapshot: &HostSnapshot,
    slot: UiContributionSlot,
    active_route: &str,
    api_base_url: &str,
    fallback: Element,
) -> Element {
    let renderers = matching_slot_renderers(snapshot, slot, active_route);
    if renderers.is_empty() {
        return fallback;
    }

    rsx! {
        for contribution in renderers {
            div { key: "{contribution.id}", class: "plugin-slot__item",
                {render_contribution(snapshot, &contribution, active_route, api_base_url)}
            }
        }
    }
}

fn render_optional_slot_panel(
    snapshot: &HostSnapshot,
    slot: UiContributionSlot,
    active_route: &str,
    api_base_url: &str,
    class_name: &str,
) -> Element {
    let renderers = matching_slot_renderers(snapshot, slot, active_route);
    if renderers.is_empty() {
        return rsx! {};
    }

    rsx! {
        aside { class: "{class_name}",
            for contribution in renderers {
                div { key: "{contribution.id}", class: "plugin-slot__item",
                    {render_contribution(snapshot, &contribution, active_route, api_base_url)}
                }
            }
        }
    }
}

fn render_contribution(
    snapshot: &HostSnapshot,
    contribution: &UiContribution,
    active_route: &str,
    api_base_url: &str,
) -> Element {
    native_renderer(snapshot, &contribution.renderer_id)
        .map(|render| {
            render(NativeRenderContext {
                active_route: active_route.to_string(),
                api_base_url: api_base_url.to_string(),
            })
        })
        .unwrap_or_else(|| EmptyPanel(contribution.label.clone(), "?".to_string()))
}

fn matching_slot_renderers(
    snapshot: &HostSnapshot,
    slot: UiContributionSlot,
    active_route: &str,
) -> Vec<UiContribution> {
    snapshot
        .ui_contributions
        .iter()
        .filter(|contribution| contribution.slot == slot)
        .filter(|contribution| {
            contribution
                .route
                .as_deref()
                .is_none_or(|route| route == active_route)
        })
        .cloned()
        .collect()
}

#[allow(non_snake_case)]
#[component]
fn TitlebarControls(sidebar_collapsed: bool, on_toggle_sidebar: EventHandler<()>) -> Element {
    let toggle_label = if sidebar_collapsed {
        "Expand sidebar"
    } else {
        "Collapse sidebar"
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
            button { class: "icon-button titlebar-nav", r#type: "button", "<" }
            button { class: "icon-button titlebar-nav", r#type: "button", ">" }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn ShellSidebar(
    nav_items: Vec<NavItemContribution>,
    settings_available: bool,
    active_route: String,
    on_route_select: EventHandler<String>,
) -> Element {
    let mut primary_items = nav_items
        .iter()
        .map(|item| {
            SidebarItemModel::primary(item.route.clone(), item.label.clone(), item.icon.clone())
        })
        .collect::<Vec<_>>();
    if settings_available {
        primary_items.push(SidebarItemModel::primary(
            SETTINGS_ROUTE,
            "Settings",
            "#".to_string(),
        ));
    }

    rsx! {
        aside { class: "sidebar",
            SidebarSectionView {
                section: SidebarSectionModel::primary(primary_items),
                active_id: active_route,
                on_select: move |route: String| on_route_select.call(route),
            }
        }
    }
}

#[allow(non_snake_case)]
fn HeaderBar() -> Element {
    rsx! {
        header { class: "header-bar",
            div { class: "header-bar__actions",
                button { class: "model-button", r#type: "button",
                    span { class: "model-button__mark", "AZ" }
                    span { "AZ AIO" }
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn EmptyPanel(title: String, mark: String) -> Element {
    rsx! {
        div { class: "empty-panel",
            div { class: "empty-panel__mark", "{mark}" }
            h1 { "{title}" }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn ShellSkeleton() -> Element {
    let rows = [0, 1, 2, 3, 4];

    rsx! {
        main { class: "az-aio-shell plugin-shell-skeleton",
            div { class: "titlebar-controls",
                div { class: "skeleton-icon" }
                div { class: "skeleton-icon skeleton-icon--small" }
                div { class: "skeleton-icon skeleton-icon--small" }
            }
            aside { class: "sidebar skeleton-sidebar",
                div { class: "sidebar__section sidebar__section--primary",
                    nav { class: "sidebar-tree sidebar-tree--primary", aria_label: "Loading navigation",
                        for row in rows {
                            div { key: "{row}", class: "skeleton-nav-row",
                                span { class: "skeleton-glyph" }
                                span { class: "skeleton-line skeleton-line--nav" }
                            }
                        }
                    }
                }
            }
            section { class: "workspace",
                header { class: "header-bar",
                    div { class: "skeleton-line skeleton-line--header" }
                }
                div { class: "workspace__body skeleton-workspace",
                    div { class: "empty-panel",
                        div { class: "skeleton-icon skeleton-icon--large" }
                        div { class: "skeleton-line skeleton-line--title" }
                    }
                }
            }
        }
    }
}
