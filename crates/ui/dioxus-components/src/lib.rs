use dioxus::prelude::*;

mod admin;

pub use admin::{
    AdminAction, AdminActionIcon, AdminCommand, AdminMenu, AdminSection, AdminShell,
    AdminShellProvider, AdminShellState, AdminTopbar, SharedAdminShellProvider,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SidebarSide {
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Default,
    Accent,
    Positive,
    Warning,
}

impl Tone {
    fn class_name(self) -> &'static str {
        match self {
            Self::Default => "",
            Self::Accent => " tone-accent",
            Self::Positive => " tone-positive",
            Self::Warning => " tone-warning",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AdminWorkbenchProps {
    pub topbar: Element,
    pub left: Element,
    pub center: Element,
    #[props(optional)]
    pub right: Option<Element>,
}

#[component]
pub fn AdminWorkbench(props: AdminWorkbenchProps) -> Element {
    let workspace_class = if props.right.is_some() {
        "workspace workspace--with-right"
    } else {
        "workspace"
    };

    rsx! {
        div { class: "admin-shell",
            {props.topbar}
            div { class: workspace_class,
                {props.left}
                {props.center}
                if let Some(right) = props.right {
                    {right}
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ThinTopbarProps {
    #[props(optional)]
    pub brand: Option<Element>,
    #[props(optional)]
    pub eyebrow: Option<String>,
    pub title: String,
    #[props(optional)]
    pub left_actions: Option<Element>,
    #[props(optional)]
    pub right_actions: Option<Element>,
}

#[component]
pub fn ThinTopbar(props: ThinTopbarProps) -> Element {
    rsx! {
        header { class: "topbar",
            div { class: "topbar__left",
                if let Some(brand) = props.brand {
                    div { class: "topbar__brand", {brand} }
                } else {
                    if let Some(eyebrow) = &props.eyebrow {
                        span { class: "topbar__eyebrow", "{eyebrow}" }
                    }
                    h1 { class: "topbar__title", "{props.title}" }
                }
                if let Some(left_actions) = props.left_actions {
                    div { class: "topbar__cluster", {left_actions} }
                }
            }
            if let Some(right_actions) = props.right_actions {
                div { class: "topbar__actions", {right_actions} }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SidebarProps {
    pub side: SidebarSide,
    pub children: Element,
}

#[component]
pub fn Sidebar(props: SidebarProps) -> Element {
    let side_class = match props.side {
        SidebarSide::Left => "sidebar sidebar--left",
        SidebarSide::Right => "sidebar sidebar--right",
    };

    rsx! {
        aside { class: side_class, {props.children} }
    }
}

#[component]
pub fn MainContent(children: Element) -> Element {
    rsx! {
        main { class: "content", {children} }
    }
}

#[component]
pub fn SidebarSection(
    label: String,
    #[props(default = false)] compact: bool,
    children: Element,
) -> Element {
    let class = if compact {
        "sidebar__section sidebar__section--compact"
    } else {
        "sidebar__section"
    };

    rsx! {
        section { class: class,
            if !compact {
                div { class: "sidebar__label", "{label}" }
            }
            {children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct WorkbenchButtonProps {
    pub class: String,
    #[props(optional)]
    pub tone: Option<Tone>,
    #[props(optional)]
    pub onclick: Option<EventHandler<MouseEvent>>,
    #[props(optional)]
    pub title: Option<String>,
    #[props(default = false)]
    pub disabled: bool,
    pub children: Element,
}

#[component]
pub fn WorkbenchButton(props: WorkbenchButtonProps) -> Element {
    let tone_class = props.tone.unwrap_or(Tone::Default).class_name();
    let class = format!("{}{}", props.class, tone_class);
    let onclick = props.onclick;
    let title = props.title;
    let disabled = props.disabled;

    rsx! {
        button {
            class: class,
            "type": "button",
            title: title,
            disabled: disabled,
            onclick: move |evt| {
                if !disabled {
                    if let Some(h) = onclick.as_ref() {
                        h.call(evt);
                    }
                }
            },
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ContentHeaderProps {
    pub title: String,
    #[props(optional)]
    pub subtitle: Option<String>,
    #[props(optional)]
    pub actions: Option<Element>,
}

#[component]
pub fn ContentHeader(props: ContentHeaderProps) -> Element {
    rsx! {
        section { class: "content-header",
            div {
                h2 { class: "content-header__title", "{props.title}" }
                if let Some(subtitle) = &props.subtitle {
                    p { class: "content-header__subtitle", "{subtitle}" }
                }
            }
            if let Some(actions) = props.actions {
                div { class: "content-header__actions", {actions} }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SectionHeaderProps {
    pub title: String,
    #[props(optional)]
    pub subtitle: Option<String>,
    #[props(optional)]
    pub eyebrow: Option<String>,
    #[props(optional)]
    pub actions: Option<Element>,
}

#[component]
pub fn SectionHeader(props: SectionHeaderProps) -> Element {
    rsx! {
        section { class: "content-header section-header",
            div {
                if let Some(eyebrow) = &props.eyebrow {
                    div { class: "section-header__eyebrow", "{eyebrow}" }
                }
                h2 { class: "content-header__title", "{props.title}" }
                if let Some(subtitle) = &props.subtitle {
                    p { class: "content-header__subtitle", "{subtitle}" }
                }
            }
            if let Some(actions) = props.actions {
                div { class: "content-header__actions", {actions} }
            }
        }
    }
}

#[component]
pub fn Surface(children: Element) -> Element {
    rsx! {
        section { class: "surface", {children} }
    }
}

#[component]
pub fn SurfaceHeader(
    title: String,
    #[props(optional)] subtitle: Option<String>,
    #[props(optional)] actions: Option<Element>,
) -> Element {
    rsx! {
        div { class: "surface__header",
            div {
                h3 { class: "surface__title", "{title}" }
                if let Some(subtitle) = &subtitle {
                    p { class: "surface__subtitle", "{subtitle}" }
                }
            }
            if let Some(actions) = actions {
                div { class: "surface__actions", {actions} }
            }
        }
    }
}

#[component]
pub fn ResponsiveGrid(columns: u8, children: Element) -> Element {
    let class = match columns {
        3 => "summary-grid",
        4 => "summary-grid summary-grid--quad",
        _ => "form-grid",
    };

    rsx! {
        div { class: class, {children} }
    }
}

#[component]
pub fn MetricStrip(columns: u8, children: Element) -> Element {
    let class = match columns {
        4 => "metric-strip metric-strip--4",
        _ => "metric-strip",
    };

    rsx! {
        div { class: class, {children} }
    }
}

#[component]
pub fn Stack(children: Element) -> Element {
    rsx! {
        div { class: "stack content-stack", {children} }
    }
}

#[component]
pub fn Divider() -> Element {
    rsx! {
        div { class: "form-divider" }
    }
}

#[component]
pub fn MetricRow(label: String, value: String, #[props(optional)] tone: Option<Tone>) -> Element {
    let tone_class = tone.unwrap_or(Tone::Default).class_name();
    let class = format!("metric-item__value{}", tone_class);

    rsx! {
        div { class: "metric-item",
            div { class: "metric-item__label", "{label}" }
            div { class: class, "{value}" }
        }
    }
}

#[component]
pub fn ListItem(
    title: String,
    #[props(optional)] detail: Option<String>,
    #[props(optional)] meta: Option<String>,
) -> Element {
    rsx! {
        div { class: "activity-item",
            div { class: "activity-item__title", "{title}" }
            if let Some(detail) = &detail {
                div { class: "activity-item__detail", "{detail}" }
            }
            if let Some(meta) = &meta {
                div { class: "activity-item__when", "{meta}" }
            }
        }
    }
}

#[component]
pub fn StatTile(
    label: String,
    value: String,
    #[props(optional)] detail: Option<String>,
    #[props(optional)] leading: Option<Element>,
) -> Element {
    rsx! {
        div { class: "summary-block",
            if let Some(leading) = leading {
                div { class: "summary-block__icon", {leading} }
            }
            div { class: "summary-block__body",
                div { class: "summary-block__label", "{label}" }
                div { class: "summary-block__value", "{value}" }
                if let Some(detail) = &detail {
                    div { class: "summary-block__detail", "{detail}" }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DataTableProps {
    pub columns: Vec<String>,
    pub children: Element,
    #[props(optional)]
    pub empty: Option<Element>,
}

#[component]
pub fn DataTable(props: DataTableProps) -> Element {
    rsx! {
        table { class: "data-table",
            thead {
                tr {
                    for column in props.columns.iter() {
                        th { "{column}" }
                    }
                }
            }
            tbody {
                {props.children}
                if let Some(empty) = props.empty {
                    {empty}
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct WorkbenchTabItemProps {
    pub label: String,
    #[props(optional)]
    pub href: Option<String>,
    #[props(default = false)]
    pub active: bool,
}

#[component]
pub fn WorkbenchTabItem(props: WorkbenchTabItemProps) -> Element {
    let tone = if props.active {
        Some(Tone::Accent)
    } else {
        None
    };
    let label = props.label.clone();

    if let Some(href) = props.href.clone() {
        rsx! {
            a { class: "workbench-tabs__link", href: href,
                WorkbenchButton { class: "segment-button".to_string(), tone: tone, "{label}" }
            }
        }
    } else {
        rsx! {
            WorkbenchButton { class: "segment-button".to_string(), tone: tone, "{label}" }
        }
    }
}

#[component]
pub fn WorkbenchTabs(children: Element) -> Element {
    rsx! {
        div { class: "knowledge-tabs workbench-tabs", {children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct GroupedListPanelProps {
    pub title: String,
    #[props(optional)]
    pub subtitle: Option<String>,
    #[props(optional)]
    pub children: Option<Element>,
    pub groups: Vec<GroupedListPanelGroup>,
}

#[derive(Clone, PartialEq)]
pub struct GroupedListPanelGroup {
    pub label: String,
    pub count_label: Option<String>,
    pub description: Option<String>,
    pub items: Vec<GroupedListPanelItem>,
}

#[derive(Clone, PartialEq)]
pub struct GroupedListPanelItem {
    pub key: String,
    pub title: String,
    pub eyebrow: Option<String>,
    pub preview: Option<String>,
    pub meta: Vec<String>,
    pub active: bool,
    pub onpress: EventHandler<MouseEvent>,
}

#[component]
pub fn GroupedListPanel(props: GroupedListPanelProps) -> Element {
    rsx! {
        Surface {
            SurfaceHeader {
                title: props.title,
                subtitle: props.subtitle
            }
            if let Some(children) = props.children {
                {children}
            }
            for group in props.groups.iter() {
                div { class: "knowledge-group",
                    div { class: "knowledge-group__label",
                        span { "{group.label}" }
                        if let Some(count_label) = &group.count_label {
                            span { class: "knowledge-group__count", "{count_label}" }
                        }
                    }
                    if let Some(description) = &group.description {
                        div { class: "callout callout--info",
                            "{description}"
                        }
                    }
                    for item in group.items.iter() {
                        button {
                            key: "{item.key}",
                            type: "button",
                            class: if item.active {
                                "knowledge-doc knowledge-doc--active"
                            } else {
                                "knowledge-doc"
                            },
                            onclick: {
                                let onpress = item.onpress;
                                move |evt| onpress.call(evt)
                            },
                            if item.eyebrow.is_some() {
                                div { class: "knowledge-doc__eyebrow",
                                    if let Some(eyebrow) = &item.eyebrow {
                                        span { class: "badge", "{eyebrow}" }
                                    }
                                }
                            }
                            div { class: "knowledge-doc__title", "{item.title}" }
                            if let Some(preview) = &item.preview {
                                div { class: "knowledge-doc__preview", "{preview}" }
                            }
                            if !item.meta.is_empty() {
                                div { class: "knowledge-doc__meta",
                                    for meta in item.meta.iter() {
                                        span { "{meta}" }
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

#[component]
pub fn TabStrip(children: Element) -> Element {
    rsx! {
        div { class: "form-tabs", {children} }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct FieldProps {
    pub label: String,
    pub value: String,
    #[props(optional)]
    pub readonly: Option<bool>,
    #[props(optional)]
    pub input_type: Option<String>,
    #[props(optional)]
    pub on_input: Option<EventHandler<String>>,
    #[props(optional)]
    pub placeholder: Option<String>,
}

#[component]
pub fn Field(props: FieldProps) -> Element {
    let readonly = props.readonly.unwrap_or(false);
    let on_input = props.on_input;
    let placeholder = props.placeholder.clone().unwrap_or_default();
    let input_type = props
        .input_type
        .clone()
        .unwrap_or_else(|| "text".to_string());
    rsx! {
        label { class: "field",
            span { class: "field__label", "{props.label}" }
            input {
                class: "field__input",
                r#type: "{input_type}",
                value: "{props.value}",
                readonly: readonly,
                placeholder: "{placeholder}",
                oninput: move |evt| {
                    if let Some(h) = on_input.as_ref() {
                        h.call(evt.value());
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TextareaProps {
    pub label: String,
    pub value: String,
    #[props(optional)]
    pub rows: Option<u8>,
    #[props(optional)]
    pub placeholder: Option<String>,
    #[props(optional)]
    pub monospace: Option<bool>,
    #[props(optional)]
    pub on_input: Option<EventHandler<String>>,
}

#[component]
pub fn Textarea(props: TextareaProps) -> Element {
    let rows = props.rows.unwrap_or(6);
    let mono = props.monospace.unwrap_or(false);
    let class = if mono {
        "textarea__input textarea__input--mono"
    } else {
        "textarea__input"
    };
    let on_input = props.on_input;
    let placeholder = props.placeholder.clone().unwrap_or_default();
    rsx! {
        label { class: "textarea",
            span { class: "field__label", "{props.label}" }
            textarea {
                class: class,
                rows: "{rows}",
                placeholder: "{placeholder}",
                value: "{props.value}",
                oninput: move |evt| {
                    if let Some(h) = on_input.as_ref() {
                        h.call(evt.value());
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct KeywordChipsProps {
    pub value: Vec<String>,
    #[props(optional)]
    pub placeholder: Option<String>,
    pub on_change: EventHandler<Vec<String>>,
}

#[component]
pub fn KeywordChips(props: KeywordChipsProps) -> Element {
    let mut draft = use_signal(String::new);
    let placeholder = props
        .placeholder
        .clone()
        .unwrap_or_else(|| "回车或逗号添加关键词".to_string());

    let mut commit_now = {
        let on_change = props.on_change;
        let current = props.value.clone();
        move |raw: String| {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return;
            }
            let mut next = current.clone();
            for token in trimmed
                .split([',', '，', '、', '/'])
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
            {
                if !next.iter().any(|k| k == token) {
                    next.push(token.to_string());
                }
            }
            on_change.call(next);
            draft.set(String::new());
        }
    };

    rsx! {
        div { class: "chips",
            for (idx, keyword) in props.value.iter().enumerate() {
                {
                    let keyword = keyword.clone();
                    let on_change = props.on_change;
                    let current = props.value.clone();
                    rsx! {
                        span { class: "chip", key: "{idx}",
                            span { class: "chip__label", "{keyword}" }
                            button {
                                class: "chip__remove",
                                "type": "button",
                                onclick: move |_| {
                                    let mut next = current.clone();
                                    next.retain(|k| k != &keyword);
                                    on_change.call(next);
                                },
                                "×"
                            }
                        }
                    }
                }
            }
            input {
                class: "chip-input",
                placeholder: "{placeholder}",
                value: "{draft}",
                oninput: {
                    let mut commit_now = commit_now.clone();
                    move |evt: FormEvent| {
                        let raw = evt.value();
                        if raw.contains(',') || raw.contains('，') || raw.contains('、') {
                            commit_now(raw);
                        } else {
                            draft.set(raw);
                        }
                    }
                },
                onkeydown: {
                    let mut commit_now = commit_now.clone();
                    move |evt: KeyboardEvent| {
                        if matches!(evt.key(), Key::Enter) {
                            evt.prevent_default();
                            commit_now(draft.read().clone());
                        }
                    }
                },
                onblur: move |_| commit_now(draft.read().clone()),
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct BadgeProps {
    pub label: String,
    #[props(optional)]
    pub variant: Option<String>,
}

#[component]
pub fn Badge(props: BadgeProps) -> Element {
    let variant = props
        .variant
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let class = format!("badge badge--{variant}");
    rsx! {
        span { class: class, "{props.label}" }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ConfirmDialogProps {
    pub open: bool,
    pub title: String,
    #[props(optional)]
    pub message: Option<String>,
    pub on_confirm: EventHandler<()>,
    pub on_cancel: EventHandler<()>,
    #[props(optional)]
    pub confirm_label: Option<String>,
    #[props(optional)]
    pub cancel_label: Option<String>,
}

#[component]
pub fn ConfirmDialog(props: ConfirmDialogProps) -> Element {
    if !props.open {
        return rsx! {};
    }
    let confirm_label = props
        .confirm_label
        .clone()
        .unwrap_or_else(|| "确认".to_string());
    let cancel_label = props
        .cancel_label
        .clone()
        .unwrap_or_else(|| "取消".to_string());
    let message = props.message.clone();
    let on_confirm = props.on_confirm;
    let on_cancel = props.on_cancel;

    rsx! {
        div { class: "dialog",
            div { class: "dialog__backdrop",
                onclick: move |_| on_cancel.call(())
            }
            div { class: "dialog__panel",
                h3 { class: "dialog__title", "{props.title}" }
                if let Some(message) = message {
                    p { class: "dialog__message", "{message}" }
                }
                div { class: "dialog__actions",
                    button {
                        class: "dialog__button",
                        "type": "button",
                        onclick: move |_| on_cancel.call(()),
                        "{cancel_label}"
                    }
                    button {
                        class: "dialog__button dialog__button--danger",
                        "type": "button",
                        onclick: move |_| on_confirm.call(()),
                        "{confirm_label}"
                    }
                }
            }
        }
    }
}
