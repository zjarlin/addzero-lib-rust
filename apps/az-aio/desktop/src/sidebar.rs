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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidebarItemModel {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
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
        }
    }
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
    let nav_class_name = section.nav_class_name.clone();
    let aria_label = section.aria_label();
    let items = section.items.clone();

    rsx! {
        div { class: "{class_name}",
            if let Some(heading) = heading.as_ref() {
                p { class: "sidebar__heading", "{heading}" }
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
fn SidebarItemButton(
    item: SidebarItemModel,
    selected: bool,
    on_select: EventHandler<String>,
) -> Element {
    let item_id = item.id.clone();
    let class_name = if selected {
        "nav-button nav-button--active"
    } else {
        "nav-button"
    };

    rsx! {
        button {
            class: "{class_name}",
            r#type: "button",
            onclick: move |_| on_select.call(item_id.clone()),
            if let Some(icon) = item.icon.as_ref() {
                span { class: "nav-button__icon", "{icon}" }
            }
            span { class: "nav-button__label", "{item.label}" }
        }
    }
}

impl SidebarSectionModel {
    fn aria_label(&self) -> String {
        self.heading.clone().unwrap_or_else(|| "Navigation".to_string())
    }
}
