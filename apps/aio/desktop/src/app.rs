use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use az_config_center_contract::{
    DEFAULT_SHELL_OUTPUT_PATH, ShellComponentBuildRequest, ShellComponentBuildResult,
    ShellComponentKind, ShellComponentPatch, ShellComponentRegistry, ShellComponentUpsert,
};
use gpui::{
    App, Application, Bounds, Context, Entity, IntoElement, Render, SharedString, Subscription,
    Window, WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use gpui_component::{
    ActiveTheme as _, Root, Selectable as _, Theme, ThemeMode,
    button::{Button, ButtonVariants as _},
    input::{Input, InputEvent, InputState},
    switch::Switch,
};
use rust_i18n::t;

use crate::{DEFAULT_LOCALE, embedded_backend::DesktopRuntime};

pub fn run() -> Result<()> {
    let runtime = DesktopRuntime::start()?;
    let runtime = Rc::new(RefCell::new(Some(runtime)));

    Application::new().run(move |cx: &mut App| {
        gpui_component::init(cx);
        Theme::change(ThemeMode::Light, None, cx);
        rust_i18n::set_locale(DEFAULT_LOCALE);
        let runtime = runtime.clone();
        let bounds = Bounds::centered(None, size(px(1440.), px(920.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |window, cx| {
                let runtime = runtime
                    .borrow_mut()
                    .take()
                    .expect("desktop runtime should only initialize once");
                let view = cx.new(|cx| ConfigCenterApp::new(runtime, window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .expect("open aio desktop window");
        cx.activate(true);
    });

    Ok(())
}

struct ConfigCenterApp {
    runtime: DesktopRuntime,
    locale: AppLocale,
    registry: ShellComponentRegistry,
    draft: ShellComponentUpsert,
    selected_name: Option<String>,
    preview_result: Option<ShellComponentBuildResult>,
    notice: String,
    name_input: Entity<InputState>,
    summary_input: Entity<InputState>,
    value_input: Entity<InputState>,
    command_input: Entity<InputState>,
    body_input: Entity<InputState>,
    output_path_input: Entity<InputState>,
    preview_input: Entity<InputState>,
    subscriptions: Vec<Subscription>,
}

impl ConfigCenterApp {
    fn new(runtime: DesktopRuntime, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let name_input = cx.new(|cx| InputState::new(window, cx));
        let summary_input = cx.new(|cx| InputState::new(window, cx));
        let value_input = cx.new(|cx| InputState::new(window, cx));
        let command_input = cx.new(|cx| InputState::new(window, cx));
        let body_input = cx.new(|cx| InputState::new(window, cx).multi_line(true));
        let output_path_input =
            cx.new(|cx| InputState::new(window, cx).default_value(DEFAULT_SHELL_OUTPUT_PATH));
        let preview_input = cx.new(|cx| InputState::new(window, cx).multi_line(true));

        let mut app = Self {
            runtime,
            locale: AppLocale::default(),
            registry: ShellComponentRegistry::default(),
            draft: blank_draft(),
            selected_name: None,
            preview_result: None,
            notice: String::new(),
            name_input,
            summary_input,
            value_input,
            command_input,
            body_input,
            output_path_input,
            preview_input,
            subscriptions: Vec::new(),
        };
        app.install_input_subscriptions(cx);
        app.reload_registry(window, cx);
        app
    }

    fn install_input_subscriptions(&mut self, cx: &mut Context<Self>) {
        self.subscriptions.push(cx.subscribe(
            &self.name_input,
            |this, input, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.draft.name = input.read(cx).value().to_string();
                    cx.notify();
                }
            },
        ));
        self.subscriptions.push(cx.subscribe(
            &self.summary_input,
            |this, input, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.draft.summary = input.read(cx).value().to_string();
                    cx.notify();
                }
            },
        ));
        self.subscriptions.push(cx.subscribe(
            &self.value_input,
            |this, input, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.draft.export_value = into_optional_string(input.read(cx).value());
                    cx.notify();
                }
            },
        ));
        self.subscriptions.push(cx.subscribe(
            &self.command_input,
            |this, input, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.draft.alias_command = into_optional_string(input.read(cx).value());
                    cx.notify();
                }
            },
        ));
        self.subscriptions.push(cx.subscribe(
            &self.body_input,
            |this, input, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.draft.body = into_optional_string(input.read(cx).value());
                    cx.notify();
                }
            },
        ));
    }

    fn reload_registry(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.runtime.client().list_shell_components() {
            Ok(registry) => {
                let selected_name = self
                    .selected_name
                    .clone()
                    .filter(|name| {
                        registry
                            .components
                            .iter()
                            .any(|component| component.name == *name)
                    })
                    .or_else(|| {
                        registry
                            .components
                            .first()
                            .map(|component| component.name.clone())
                    });
                let output_path = registry.build.output_path.clone();
                self.registry = registry;
                self.notice = t!(
                    "notice.connected",
                    base_url = self.runtime.base_url().to_string()
                )
                .to_string();
                self.set_input_value(&self.output_path_input, output_path, window, cx);
                if let Some(selected_name) = selected_name {
                    self.select_component(selected_name, window, cx);
                } else {
                    self.reset_draft(window, cx);
                }
            }
            Err(err) => {
                self.notice = err.to_string();
            }
        }
    }

    fn select_component(&mut self, name: String, window: &mut Window, cx: &mut Context<Self>) {
        let Some(component) = self
            .registry
            .components
            .iter()
            .find(|component| component.name == name)
            .cloned()
        else {
            return;
        };
        self.selected_name = Some(component.name.clone());
        self.draft = ShellComponentUpsert {
            name: component.name.clone(),
            kind: component.kind,
            summary: component.summary.clone(),
            enabled: component.enabled,
            render_to_output: component.render_to_output,
            export_value: component.export_value.clone(),
            alias_command: component.alias_command.clone(),
            body: component.body.clone(),
        };
        self.sync_editor_inputs(window, cx);
        self.set_preview_text(component.preview, window, cx);
        cx.notify();
    }

    fn reset_draft(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.selected_name = None;
        self.draft = blank_draft();
        self.sync_editor_inputs(window, cx);
        self.set_preview_text(String::new(), window, cx);
        cx.notify();
    }

    fn sync_editor_inputs(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_input_value(&self.name_input, self.draft.name.clone(), window, cx);
        self.set_input_value(&self.summary_input, self.draft.summary.clone(), window, cx);
        self.set_input_value(
            &self.value_input,
            self.draft.export_value.clone().unwrap_or_default(),
            window,
            cx,
        );
        self.set_input_value(
            &self.command_input,
            self.draft.alias_command.clone().unwrap_or_default(),
            window,
            cx,
        );
        self.set_input_value(
            &self.body_input,
            self.draft.body.clone().unwrap_or_default(),
            window,
            cx,
        );
    }

    fn set_input_value(
        &self,
        input: &Entity<InputState>,
        value: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = value.into();
        input.update(cx, |state, cx| state.set_value(value.clone(), window, cx));
    }

    fn set_preview_text(&self, value: String, window: &mut Window, cx: &mut Context<Self>) {
        self.preview_input
            .update(cx, |state, cx| state.set_value(value, window, cx));
    }

    fn save_component(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.runtime.client().upsert_shell_component(&self.draft) {
            Ok(component) => {
                self.notice =
                    t!("notice.component_saved", name = component.name.as_str()).to_string();
                self.selected_name = Some(component.name.clone());
                self.reload_registry(window, cx);
                self.select_component(component.name, window, cx);
            }
            Err(err) => {
                self.notice = err.to_string();
            }
        }
    }

    fn delete_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(name) = self.selected_name.clone() else {
            self.notice = t!("notice.select_before_delete").to_string();
            return;
        };
        match self
            .runtime
            .client()
            .remove_shell_component(&az_config_center_contract::ShellComponentRemove { name })
        {
            Ok(component) => {
                self.notice =
                    t!("notice.component_removed", name = component.name.as_str()).to_string();
                self.reload_registry(window, cx);
            }
            Err(err) => {
                self.notice = err.to_string();
            }
        }
    }

    fn save_output_path(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let output_path = self.output_path_input.read(cx).value().to_string();
        match self.runtime.client().save_shell_component_config(
            &az_config_center_contract::ShellComponentConfigUpdate {
                output_path: Some(output_path.clone()),
            },
        ) {
            Ok(registry) => {
                self.registry = registry;
                self.notice = t!("notice.output_path_saved", path = output_path.trim()).to_string();
            }
            Err(err) => {
                self.notice = err.to_string();
            }
        }
    }

    fn preview_build(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.run_build(false, window, cx);
    }

    fn apply_build(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.run_build(true, window, cx);
    }

    fn run_build(&mut self, write: bool, window: &mut Window, cx: &mut Context<Self>) {
        let output_path = self.output_path_input.read(cx).value().to_string();
        match self
            .runtime
            .client()
            .build_shell_components(&ShellComponentBuildRequest {
                output_path: into_optional_string(output_path.into()),
                write,
            }) {
            Ok(result) => {
                self.notice = if write {
                    t!("notice.output_applied", path = result.output_path.as_str()).to_string()
                } else {
                    t!(
                        "notice.preview_ready",
                        count = result.included_components.to_string()
                    )
                    .to_string()
                };
                self.preview_result = Some(result.clone());
                self.set_preview_text(result.content.clone(), window, cx);
            }
            Err(err) => {
                self.notice = err.to_string();
            }
        }
    }

    fn patch_flags(
        &mut self,
        enabled: Option<bool>,
        render_to_output: Option<bool>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(value) = enabled {
            self.draft.enabled = value;
        }
        if let Some(value) = render_to_output {
            self.draft.render_to_output = value;
        }

        let Some(name) = self.selected_name.clone() else {
            cx.notify();
            return;
        };

        match self
            .runtime
            .client()
            .patch_shell_component(&ShellComponentPatch {
                name,
                summary: None,
                enabled,
                render_to_output,
            }) {
            Ok(component) => {
                self.notice = t!(
                    "notice.component_flags_updated",
                    name = component.name.as_str()
                )
                .to_string();
                self.reload_registry(window, cx);
                self.select_component(component.name, window, cx);
            }
            Err(err) => {
                self.notice = err.to_string();
            }
        }
    }

    fn switch_locale(&mut self, locale: AppLocale, cx: &mut Context<Self>) {
        self.locale = locale;
        rust_i18n::set_locale(locale.code());
        self.notice = t!("notice.locale_switched", language = locale.native_name()).to_string();
        cx.notify();
    }

    fn render_locale_picker(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .children(AppLocale::ALL.into_iter().map(|locale| {
                let is_selected = self.locale == locale;
                Button::new(("locale", locale.id()))
                    .label(locale.native_name())
                    .selected(is_selected)
                    .when(is_selected, |button| button.primary())
                    .when(!is_selected, |button| button.ghost())
                    .on_click(cx.listener(move |this, _, _, cx| this.switch_locale(locale, cx)))
            }))
    }

    fn render_toolbar(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        div()
            .flex()
            .gap_3()
            .items_center()
            .px_4()
            .py_3()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.background)
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(t!("toolbar.language").to_string()),
            )
            .child(self.render_locale_picker(cx))
            .child(
                div()
                    .w(px(96.))
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(t!("toolbar.output_path").to_string()),
            )
            .child(Input::new(&self.output_path_input).w_full())
            .child(
                Button::new("save-output-path")
                    .label(t!("toolbar.save_path").to_string())
                    .on_click(cx.listener(|this, _, window, cx| this.save_output_path(window, cx))),
            )
            .child(
                Button::new("preview-build")
                    .label(t!("toolbar.preview").to_string())
                    .primary()
                    .on_click(cx.listener(|this, _, window, cx| this.preview_build(window, cx))),
            )
            .child(
                Button::new("apply-build")
                    .label(t!("toolbar.apply_output").to_string())
                    .success()
                    .on_click(cx.listener(|this, _, window, cx| this.apply_build(window, cx))),
            )
    }

    fn render_component_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let selected_name = self.selected_name.clone();
        div()
            .w(px(300.))
            .h_full()
            .flex()
            .flex_col()
            .border_r_1()
            .border_color(theme.border)
            .bg(theme.sidebar)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_3()
                    .border_b_1()
                    .border_color(theme.sidebar_border)
                    .child(t!("list.title").to_string())
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                Button::new("new-component")
                                    .label(t!("list.new").to_string())
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.reset_draft(window, cx)
                                    })),
                            )
                            .child(
                                Button::new("refresh-components")
                                    .label(t!("list.refresh").to_string())
                                    .ghost()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.reload_registry(window, cx)
                                    })),
                            ),
                    ),
            )
            .children(
                self.registry
                    .components
                    .iter()
                    .enumerate()
                    .map(|(index, component)| {
                        let is_selected = selected_name.as_ref() == Some(&component.name);
                        let name = component.name.clone();
                        let subtitle = t!(
                            "component.subtitle",
                            kind = kind_label(component.kind),
                            enabled = component_enabled_label(component.enabled),
                            render = render_output_label(component.render_to_output)
                        )
                        .to_string();
                        Button::new(("component", index))
                            .ghost()
                            .selected(is_selected)
                            .w_full()
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.select_component(name.clone(), window, cx)
                            }))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .items_start()
                                    .w_full()
                                    .py_2()
                                    .child(component.name.clone())
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme.muted_foreground)
                                            .child(subtitle),
                                    ),
                            )
                    }),
            )
    }

    fn render_kind_picker(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .gap_2()
            .children(ShellComponentKind::ALL.into_iter().map(|kind| {
                let is_selected = self.draft.kind == kind;
                Button::new(kind.code())
                    .label(kind_label(kind))
                    .selected(is_selected)
                    .when(is_selected, |button| button.primary())
                    .when(!is_selected, |button| button.ghost())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.draft.kind = kind;
                        cx.notify();
                    }))
            }))
    }

    fn render_editor(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_3()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(t!("editor.title").to_string())
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                Button::new("save-component")
                                    .label(t!("editor.save").to_string())
                                    .primary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.save_component(window, cx)
                                    })),
                            )
                            .child(
                                Button::new("delete-component")
                                    .label(t!("editor.delete").to_string())
                                    .danger()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.delete_selected(window, cx)
                                    })),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .child(field_row(
                        t!("field.name").to_string(),
                        Input::new(&self.name_input).w_full(),
                    ))
                    .child(field_row(
                        t!("field.summary").to_string(),
                        Input::new(&self.summary_input).w_full(),
                    ))
                    .child(field_row(
                        t!("field.kind").to_string(),
                        self.render_kind_picker(cx),
                    ))
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .items_center()
                            .child(
                                Switch::new("enabled-switch")
                                    .checked(self.draft.enabled)
                                    .label(t!("field.enabled").to_string())
                                    .on_click(cx.listener(|this, checked: &bool, window, cx| {
                                        this.patch_flags(Some(*checked), None, window, cx)
                                    })),
                            )
                            .child(
                                Switch::new("render-switch")
                                    .checked(self.draft.render_to_output)
                                    .label(t!("field.render_to_output").to_string())
                                    .on_click(cx.listener(|this, checked: &bool, window, cx| {
                                        this.patch_flags(None, Some(*checked), window, cx)
                                    })),
                            ),
                    )
                    .when(
                        matches!(self.draft.kind, ShellComponentKind::Export),
                        |this| {
                            this.child(field_row(
                                t!("field.export_value").to_string(),
                                Input::new(&self.value_input).w_full(),
                            ))
                        },
                    )
                    .when(
                        matches!(self.draft.kind, ShellComponentKind::Alias),
                        |this| {
                            this.child(field_row(
                                t!("field.alias_command").to_string(),
                                Input::new(&self.command_input).w_full(),
                            ))
                        },
                    )
                    .when(
                        matches!(
                            self.draft.kind,
                            ShellComponentKind::Function | ShellComponentKind::Snippet
                        ),
                        |this| {
                            this.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(theme.muted_foreground)
                                            .child(t!("field.body").to_string()),
                                    )
                                    .child(
                                        div()
                                            .h(px(320.))
                                            .child(Input::new(&self.body_input).h_full().w_full()),
                                    ),
                            )
                        },
                    ),
            )
    }

    fn render_preview(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        div()
            .w(px(480.))
            .h_full()
            .flex()
            .flex_col()
            .border_l_1()
            .border_color(theme.border)
            .bg(theme.background)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_4()
                    .py_3()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(t!("preview.title").to_string())
                    .child(
                        div().text_sm().text_color(theme.muted_foreground).child(
                            self.preview_result
                                .as_ref()
                                .map(|result| {
                                    t!(
                                        "preview.summary",
                                        included = result.included_components.to_string(),
                                        total = result.total_components.to_string()
                                    )
                                    .to_string()
                                })
                                .unwrap_or_else(|| t!("preview.empty").to_string()),
                        ),
                    ),
            )
            .child(
                div().flex_1().p_4().child(
                    Input::new(&self.preview_input)
                        .h_full()
                        .w_full()
                        .disabled(true)
                        .selected(true),
                ),
            )
            .child(
                div()
                    .px_4()
                    .py_3()
                    .border_t_1()
                    .border_color(theme.border)
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(self.notice.clone()),
            )
    }
}

impl Render for ConfigCenterApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        div()
            .size_full()
            .bg(theme.background)
            .text_color(theme.foreground)
            .flex()
            .flex_col()
            .child(self.render_toolbar(window, cx))
            .child(
                div()
                    .flex_1()
                    .size_full()
                    .flex()
                    .child(self.render_component_list(cx))
                    .child(self.render_editor(window, cx))
                    .child(self.render_preview(window, cx)),
            )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum AppLocale {
    #[default]
    ZhCn,
    En,
}

impl AppLocale {
    const ALL: [Self; 2] = [Self::ZhCn, Self::En];

    fn code(self) -> &'static str {
        match self {
            Self::ZhCn => "zh-CN",
            Self::En => "en",
        }
    }

    fn native_name(self) -> &'static str {
        match self {
            Self::ZhCn => "中文",
            Self::En => "English",
        }
    }

    fn id(self) -> u32 {
        match self {
            Self::ZhCn => 0,
            Self::En => 1,
        }
    }
}

fn blank_draft() -> ShellComponentUpsert {
    ShellComponentUpsert {
        name: String::new(),
        kind: ShellComponentKind::Snippet,
        summary: String::new(),
        enabled: true,
        render_to_output: true,
        export_value: None,
        alias_command: None,
        body: None,
    }
}

fn field_row(label: impl Into<String>, element: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap_3()
        .child(
            div()
                .w(px(120.))
                .text_sm()
                .text_color(gpui::rgb(0x667085))
                .child(label.into()),
        )
        .child(div().flex_1().child(element))
}

fn into_optional_string(value: SharedString) -> Option<String> {
    let value = value.to_string();
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn kind_label(kind: ShellComponentKind) -> String {
    match kind {
        ShellComponentKind::Export => t!("kind.export").to_string(),
        ShellComponentKind::Alias => t!("kind.alias").to_string(),
        ShellComponentKind::Function => t!("kind.function").to_string(),
        ShellComponentKind::Snippet => t!("kind.snippet").to_string(),
    }
}

fn component_enabled_label(enabled: bool) -> String {
    if enabled {
        t!("component.enabled").to_string()
    } else {
        t!("component.disabled").to_string()
    }
}

fn render_output_label(render_to_output: bool) -> String {
    if render_to_output {
        t!("component.render_on").to_string()
    } else {
        t!("component.render_off").to_string()
    }
}
