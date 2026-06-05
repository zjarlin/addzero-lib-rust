#![forbid(unsafe_code)]

use dioxus::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidebarSectionModel {
    pub heading: Option<String>,
    pub class_name: String,
    pub nav_class_name: String,
    pub items: Vec<SidebarItemModel>,
}

impl SidebarSectionModel {
    pub fn primary(items: Vec<SidebarItemModel>) -> Self {
        Self {
            heading: None,
            class_name: "sidebar__section sidebar__section--primary".to_string(),
            nav_class_name: "sidebar-tree sidebar-tree--primary".to_string(),
            items,
        }
    }

    pub fn app_group(heading: impl Into<String>, items: Vec<SidebarItemModel>) -> Self {
        Self {
            heading: Some(heading.into()),
            class_name: "sidebar__section".to_string(),
            nav_class_name: "sidebar-tree".to_string(),
            items,
        }
    }

    pub fn recent(heading: impl Into<String>, items: Vec<SidebarItemModel>) -> Self {
        Self {
            heading: Some(heading.into()),
            class_name: "sidebar__section sidebar__section--recent".to_string(),
            nav_class_name: "sidebar-tree".to_string(),
            items,
        }
    }

    pub fn settings_tree(heading: impl Into<String>, items: Vec<SidebarItemModel>) -> Self {
        Self {
            heading: Some(heading.into()),
            class_name: "settings-tree__section".to_string(),
            nav_class_name: "sidebar-tree sidebar-tree--settings".to_string(),
            items,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidebarItemModel {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub detail: Option<String>,
    pub depth: u8,
    pub kind: SidebarItemKind,
}

impl SidebarItemModel {
    pub fn primary(
        id: impl Into<String>,
        label: impl Into<String>,
        icon: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: Some(icon.into()),
            detail: None,
            depth: 0,
            kind: SidebarItemKind::Primary,
        }
    }

    pub fn project(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: Some("▱".to_string()),
            detail: None,
            depth: 0,
            kind: SidebarItemKind::Project,
        }
    }

    pub fn thread(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            detail: None,
            depth: 0,
            kind: SidebarItemKind::Thread,
        }
    }

    pub fn settings_action(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: Some("⚙".to_string()),
            detail: None,
            depth: 0,
            kind: SidebarItemKind::SettingsAction,
        }
    }

    pub fn tree(
        id: impl Into<String>,
        label: impl Into<String>,
        icon: impl Into<String>,
        depth: u8,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: Some(icon.into()),
            detail: None,
            depth,
            kind: SidebarItemKind::Tree,
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SidebarItemKind {
    Primary,
    Project,
    Thread,
    SettingsAction,
    Tree,
}

#[allow(non_snake_case)]
#[component]
pub fn SidebarSectionView(
    section: SidebarSectionModel,
    active_id: String,
    on_select: EventHandler<String>,
) -> Element {
    let class_name = section.class_name.clone();
    let heading = section.heading.clone();
    let heading_class = section_heading_class(&section);
    let nav_class_name = section.nav_class_name.clone();
    let aria_label = section.aria_label();
    let items = section.items.clone();

    rsx! {
        div { class: "{class_name}",
            if let Some(heading) = heading.as_ref() {
                p { class: "{heading_class}", "{heading}" }
            }
            nav { class: "{nav_class_name}", aria_label: "{aria_label}",
                for item in items {
                    {
                        let selected = item.id == active_id;
                        rsx! {
                            SidebarItemButton {
                                item,
                                selected,
                                on_select,
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
pub fn SidebarActionButton(
    item: SidebarItemModel,
    selected: bool,
    on_select: EventHandler<String>,
) -> Element {
    rsx! {
        SidebarItemButton {
            item,
            selected,
            on_select,
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn SidebarItemButton(
    item: SidebarItemModel,
    selected: bool,
    on_select: EventHandler<String>,
) -> Element {
    let item_id = item.id.clone();
    let class_name = sidebar_item_class(item.kind, selected);
    let style = sidebar_item_style(item.depth);
    let icon_class = sidebar_icon_class(item.kind);
    let label_class = sidebar_label_class(item.kind);

    if item.kind == SidebarItemKind::Thread {
        return rsx! {
            button {
                class: "{class_name}",
                r#type: "button",
                onclick: move |_| on_select.call(item_id.clone()),
                "{item.label}"
            }
        };
    }

    rsx! {
        button {
            class: "{class_name}",
            style: "{style}",
            r#type: "button",
            onclick: move |_| on_select.call(item_id.clone()),
            if let Some(icon) = item.icon.as_ref() {
                span { class: "{icon_class}", "{icon}" }
            }
            span { class: "{label_class}", "{item.label}" }
            if let Some(detail) = item.detail.as_ref() {
                span { class: "nav-button__detail", "{detail}" }
            }
        }
    }
}

impl SidebarSectionModel {
    fn aria_label(&self) -> String {
        self.heading
            .clone()
            .unwrap_or_else(|| "主导航".to_string())
    }
}

fn section_heading_class(section: &SidebarSectionModel) -> &'static str {
    if section.class_name.starts_with("settings-tree") {
        "settings-tree__heading"
    } else {
        "sidebar__heading"
    }
}

fn sidebar_item_class(kind: SidebarItemKind, selected: bool) -> String {
    let base = match kind {
        SidebarItemKind::Primary => "nav-button",
        SidebarItemKind::Project => "project-row",
        SidebarItemKind::Thread => "thread-row",
        SidebarItemKind::SettingsAction => "settings-button",
        SidebarItemKind::Tree => "nav-button nav-button--tree",
    };

    let active = match (kind, selected) {
        (SidebarItemKind::SettingsAction, true) => " settings-button--active",
        (_, true) => " nav-button--active",
        (_, false) => "",
    };

    format!("{base}{active}")
}

fn sidebar_item_style(depth: u8) -> String {
    format!("--tree-depth: {depth};")
}

fn sidebar_icon_class(kind: SidebarItemKind) -> &'static str {
    match kind {
        SidebarItemKind::Project => "project-row__icon",
        SidebarItemKind::SettingsAction => "settings-button__icon",
        SidebarItemKind::Primary | SidebarItemKind::Tree | SidebarItemKind::Thread => {
            "nav-button__icon"
        }
    }
}

fn sidebar_label_class(kind: SidebarItemKind) -> &'static str {
    match kind {
        SidebarItemKind::Project => "project-row__label",
        SidebarItemKind::SettingsAction => "settings-button__label",
        SidebarItemKind::Primary | SidebarItemKind::Tree | SidebarItemKind::Thread => {
            "nav-button__label"
        }
    }
}
