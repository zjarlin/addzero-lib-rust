use std::{cmp::Reverse, collections::BTreeMap, sync::Arc};

use anyhow::Result;
use az_desktop_plugin::{
    DesktopBranchRegistration, DesktopEvent, DesktopExecContext, DesktopHostRegistry,
    DesktopInitContext, DesktopPageRegistration, DesktopPageRole, DesktopPlugin,
    DesktopRenderLayer, DesktopShellSnapshot, DesktopViewContext, EventPropagation,
};
use az_desktop_plugin_registry::load_plugins;
use gpui::{
    App, Application, Bounds, Context, IntoElement, Render, SharedString, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_component::{
    ActiveTheme as _, Root, Selectable as _, Theme, ThemeMode,
    button::{Button, ButtonVariants as _},
    scroll::ScrollableElement as _,
};

use crate::host_services::InProcessHostServices;

const DASHBOARD_ROUTE: &str = "/";

pub fn run() -> Result<()> {
    let services = Arc::new(InProcessHostServices::new()?);

    Application::new().run(move |cx: &mut App| {
        gpui_component::init(cx);
        Theme::change(ThemeMode::Light, None, cx);
        let services = services.clone();
        let bounds = Bounds::centered(None, size(px(1520.), px(960.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |window, cx| {
                let services = services.clone();
                let view = cx.new(|cx| DesktopHostApp::new(services, window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .expect("open aio desktop window");
        cx.activate(true);
    });

    Ok(())
}

struct DesktopHostApp {
    services: Arc<InProcessHostServices>,
    registry: DesktopHostRegistry,
    plugins: Vec<Box<DesktopPlugin>>,
    plugin_indices: BTreeMap<String, usize>,
    current_route: String,
    selected_entity: Option<String>,
    notice: String,
}

impl DesktopHostApp {
    fn new(
        services: Arc<InProcessHostServices>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut plugins = load_plugins();
        let mut init_ctx = DesktopInitContext::new();
        for plugin in &mut plugins {
            init_ctx.set_current_plugin(plugin.name());
            plugin.setup(&mut init_ctx);
        }
        let registry = DesktopHostRegistry::from(init_ctx.into_contributions());
        let plugin_indices = plugins
            .iter()
            .enumerate()
            .map(|(index, plugin)| (plugin.name().to_string(), index))
            .collect::<BTreeMap<_, _>>();

        let mut app = Self {
            services,
            registry,
            plugins,
            plugin_indices,
            current_route: DASHBOARD_ROUTE.to_string(),
            selected_entity: None,
            notice: "AIO desktop plugin host ready.".to_string(),
        };
        app.dispatch_event(DesktopEvent::Startup, cx);
        app.dispatch_event(DesktopEvent::RefreshRequested { route: None }, cx);
        app
    }

    fn shell_snapshot(&self) -> DesktopShellSnapshot {
        let page = self.registry.page_for_route(&self.current_route);
        DesktopShellSnapshot {
            current_route: self.current_route.clone(),
            current_domain_id: page.map(|page| page.domain_id.clone()),
            current_page_id: page.map(|page| page.id.clone()),
            selected_entity: self.selected_entity.clone(),
            notice: (!self.notice.trim().is_empty()).then(|| self.notice.clone()),
        }
    }

    fn navigate_to(&mut self, route: impl Into<String>, cx: &mut Context<Self>) {
        let route = route.into();
        if self.current_route == route {
            self.dispatch_event(
                DesktopEvent::RefreshRequested {
                    route: Some(route.clone()),
                },
                cx,
            );
            return;
        }

        self.current_route = route.clone();
        self.dispatch_event(DesktopEvent::RouteChanged { route }, cx);
    }

    fn dispatch_event(&mut self, event: DesktopEvent, cx: &mut Context<Self>) {
        let event_route = match &event {
            DesktopEvent::ActionInvoked { route, .. } => Some(route.clone()),
            DesktopEvent::RefreshRequested { route } => route.clone(),
            DesktopEvent::RouteChanged { route } => Some(route.clone()),
            DesktopEvent::SelectionChanged { route, .. } => Some(route.clone()),
            _ => Some(self.current_route.clone()),
        };
        let targets = self.event_targets(event_route.as_deref());
        let services: Arc<dyn az_desktop_plugin::DesktopHostServices> = self.services.clone();
        let shell = self.shell_snapshot();
        let (exec_ctx, feedback) = DesktopExecContext::new(services, shell);

        for index in targets {
            let mut ctx = exec_ctx.clone();
            let propagation = self.plugins[index].on_event(&event, &mut ctx);
            if propagation == EventPropagation::Stop {
                break;
            }
        }

        let feedback = feedback.borrow().clone();
        if let Some(notice) = feedback.notice {
            self.notice = notice;
        }
        if let Some(selected_entity) = feedback.selected_entity {
            self.selected_entity = selected_entity;
        }
        if let Some(route) = feedback.route_override {
            self.navigate_to(route, cx);
            return;
        }
        if feedback.refresh_requested {
            self.dispatch_event(
                DesktopEvent::RefreshRequested {
                    route: Some(self.current_route.clone()),
                },
                cx,
            );
            return;
        }
        cx.notify();
    }

    fn event_targets(&self, route: Option<&str>) -> Vec<usize> {
        let mut targets = Vec::new();
        if let Some(route) = route {
            let owners = self
                .registry
                .plugins_for_route(route, DesktopPageRole::Owner);
            for plugin_name in owners {
                self.push_plugin_target(&plugin_name, &mut targets);
            }

            let contributors = self
                .registry
                .plugins_for_route(route, DesktopPageRole::Contributor);
            for plugin_name in contributors {
                self.push_plugin_target(&plugin_name, &mut targets);
            }
        }

        for (name, index) in &self.plugin_indices {
            if !targets.contains(index)
                && self.plugins[*index].render_layer() == DesktopRenderLayer::Overlay
            {
                let _ = name;
                targets.push(*index);
            }
        }

        targets.sort_by_key(|index| Reverse(self.plugins[*index].priority()));
        targets
    }

    fn push_plugin_target(&self, plugin_name: &str, targets: &mut Vec<usize>) {
        if let Some(index) = self.plugin_indices.get(plugin_name).copied()
            && !targets.contains(&index)
        {
            targets.push(index);
        }
    }

    fn render_topbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let dashboard_button = if self.current_route == DASHBOARD_ROUTE {
            Button::new("dashboard-route")
                .label("Dashboard")
                .selected(true)
                .primary()
        } else {
            Button::new("dashboard-route")
                .label("Dashboard")
                .selected(false)
                .ghost()
        };
        div()
            .flex()
            .items_center()
            .gap_3()
            .px_4()
            .py_3()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.background)
            .child(dashboard_button.on_click(
                cx.listener(|this, _, _, cx| this.navigate_to(DASHBOARD_ROUTE.to_string(), cx)),
            ))
            .children(self.registry.domains().iter().map(|domain| {
                let active = self
                    .registry
                    .domain_for_route(&self.current_route)
                    .is_some_and(|current| current.id == domain.id);
                let route = domain.default_route.clone();
                let button = if active {
                    Button::new(SharedString::from(format!("domain-{}", domain.id)))
                        .label(domain.label.clone())
                        .selected(true)
                        .primary()
                } else {
                    Button::new(SharedString::from(format!("domain-{}", domain.id)))
                        .label(domain.label.clone())
                        .selected(false)
                        .ghost()
                };
                button.on_click(
                    cx.listener(move |this, _, _, cx| this.navigate_to(route.clone(), cx)),
                )
            }))
    }

    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let page_title = self
            .registry
            .page_for_route(&self.current_route)
            .map(|page| page.title.clone())
            .unwrap_or_else(|| "Host Dashboard".to_string());
        let page_subtitle = self
            .registry
            .page_for_route(&self.current_route)
            .map(|page| page.subtitle.clone())
            .unwrap_or_else(|| "Plugin summary cards and route hub".to_string());

        div()
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .py_3()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.background)
            .child(
                div().flex().flex_col().gap_1().child(page_title).child(
                    div()
                        .text_sm()
                        .text_color(theme.muted_foreground)
                        .child(page_subtitle),
                ),
            )
            .child(
                div().flex().gap_2().children(
                    self.registry
                        .toolbar_actions_for_route(&self.current_route)
                        .into_iter()
                        .map(|action| {
                            let action_id = action.action_id.clone();
                            let route = self.current_route.clone();
                            let button = Button::new(SharedString::from(format!(
                                "toolbar-action-{action_id}"
                            )))
                            .label(action.label.clone())
                            .selected(false);
                            let button = if action.primary {
                                button.primary()
                            } else {
                                button.ghost()
                            };
                            button.on_click(cx.listener(move |this, _, _, cx| {
                                this.dispatch_event(
                                    DesktopEvent::ActionInvoked {
                                        route: route.clone(),
                                        action_id: action_id.clone(),
                                    },
                                    cx,
                                )
                            }))
                        }),
                ),
            )
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let domain_id = self
            .registry
            .domain_for_route(&self.current_route)
            .map(|domain| domain.id.clone());

        div()
            .w(px(300.))
            .h_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .border_r_1()
            .border_color(theme.border)
            .bg(theme.sidebar)
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("Context Tree"),
            )
            .when_some(domain_id.clone(), |sidebar, domain_id| {
                sidebar.children(
                    self.registry
                        .root_branches_for_domain(&domain_id)
                        .into_iter()
                        .map(|branch| self.render_branch(branch, 0, cx)),
                )
            })
            .when_some(domain_id, |sidebar, domain_id| {
                let top_pages = self.registry.root_pages_for_domain(&domain_id);
                sidebar.children(
                    top_pages
                        .into_iter()
                        .map(|page| self.render_page_button(page, 0, cx)),
                )
            })
    }

    fn render_branch(
        &self,
        branch: &DesktopBranchRegistration,
        depth: usize,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        div()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .pl(px((depth as f32) * 12.0))
                    .text_sm()
                    .text_color(rgb(0x475467))
                    .child(branch.label.clone()),
            )
            .children(
                self.registry
                    .pages_for_branch(&branch.id)
                    .into_iter()
                    .map(|page| self.render_page_button(page, depth + 1, cx)),
            )
            .children(
                self.registry
                    .child_branches(&branch.id)
                    .into_iter()
                    .map(|child| self.render_branch(child, depth + 1, cx)),
            )
            .into_any_element()
    }

    fn render_page_button(
        &self,
        page: &DesktopPageRegistration,
        depth: usize,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let route = page.route.clone();
        let active = self.current_route == route;
        Button::new(SharedString::from(format!("route-{route}")))
            .w_full()
            .ghost()
            .selected(active)
            .on_click(cx.listener(move |this, _, _, cx| this.navigate_to(route.clone(), cx)))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_start()
                    .pl(px((depth as f32) * 12.0))
                    .py_2()
                    .child(page.title.clone())
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x667085))
                            .child(page.subtitle.clone()),
                    ),
            )
            .into_any_element()
    }

    fn render_dashboard(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p_6()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_xl().child("AIO Desktop"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x475467))
                            .child("Host dashboard powered by plugin summary cards."),
                    ),
            )
            .children(self.registry.summary_cards().iter().map(|card| {
                let route = card.route.clone();
                Button::new(SharedString::from(format!("summary-card-{}", card.card_id)))
                    .w_full()
                    .ghost()
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.navigate_to(route.clone(), cx)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_start()
                            .gap_1()
                            .w_full()
                            .py_3()
                            .child(card.title.clone())
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x475467))
                                    .child(card.summary.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x667085))
                                    .child(card.plugin_name.clone()),
                            ),
                    )
            }))
    }
}

impl Render for DesktopHostApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let mut main_ctx = DesktopViewContext {
            shell: self.shell_snapshot(),
        };
        let mut inspector_ctx = main_ctx.clone();
        let mut overlay_ctx = main_ctx.clone();

        let mut main_elements = self
            .registry
            .plugins_for_render_layer(
                &self.current_route,
                DesktopRenderLayer::Main,
                &self.plugins,
                &self.plugin_indices,
            )
            .into_iter()
            .filter_map(|index| self.plugins[index].render(&mut main_ctx))
            .collect::<Vec<_>>();

        let inspector_elements = self
            .registry
            .plugins_for_render_layer(
                &self.current_route,
                DesktopRenderLayer::Inspector,
                &self.plugins,
                &self.plugin_indices,
            )
            .into_iter()
            .filter_map(|index| self.plugins[index].render(&mut inspector_ctx))
            .collect::<Vec<_>>();

        let overlay_elements = self
            .registry
            .plugins_for_render_layer(
                &self.current_route,
                DesktopRenderLayer::Overlay,
                &self.plugins,
                &self.plugin_indices,
            )
            .into_iter()
            .filter_map(|index| self.plugins[index].render(&mut overlay_ctx))
            .collect::<Vec<_>>();

        if self.current_route == DASHBOARD_ROUTE {
            main_elements = vec![self.render_dashboard(cx).into_any_element()];
        }

        let page = self.registry.page_for_route(&self.current_route);

        div()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .flex()
            .flex_col()
            .child(self.render_topbar(cx))
            .child(self.render_toolbar(cx))
            .child(
                div()
                    .flex_1()
                    .size_full()
                    .flex()
                    .child(self.render_sidebar(cx))
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .overflow_y_scrollbar()
                            .children(main_elements),
                    )
                    .child(
                        div()
                            .w(px(320.))
                            .h_full()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .p_4()
                            .border_l_1()
                            .border_color(theme.border)
                            .bg(theme.background)
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child("Inspector"),
                            )
                            .child(div().text_sm().child(format!(
                                "Route: {}",
                                if self.current_route == DASHBOARD_ROUTE {
                                    "/"
                                } else {
                                    &self.current_route
                                }
                            )))
                            .when_some(page, |panel, page| {
                                panel.child(
                                    div()
                                        .text_sm()
                                        .text_color(theme.muted_foreground)
                                        .child(page.subtitle.clone()),
                                )
                            })
                            .when_some(self.selected_entity.clone(), |panel, entity| {
                                panel.child(
                                    div()
                                        .text_sm()
                                        .text_color(theme.muted_foreground)
                                        .child(format!("Selected: {entity}")),
                                )
                            })
                            .children(inspector_elements),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .px_4()
                    .py_3()
                    .border_t_1()
                    .border_color(theme.border)
                    .bg(theme.background)
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(self.notice.clone()),
                    )
                    .children(overlay_elements),
            )
    }
}
