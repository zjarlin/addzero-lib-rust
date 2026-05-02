use dioxus::prelude::*;
use dioxus_free_icons::{
    Icon,
    icons::ld_icons::{LdBell, LdLogOut, LdMoonStar, LdSearch, LdSun},
};
use std::rc::Rc;

use crate::{
    AdminAction, AdminActionIcon, AdminMenu, AdminSection, AdminWorkbench, MainContent,
    SharedAdminShellProvider, Sidebar, SidebarSection, SidebarSide, ThinTopbar, WorkbenchButton,
};

#[derive(Props, Clone)]
pub struct AdminShellProps<R>
where
    R: Routable + Clone + PartialEq + 'static,
{
    pub provider: SharedAdminShellProvider<R>,
    pub content: Element,
}

impl<R> PartialEq for AdminShellProps<R>
where
    R: Routable + Clone + PartialEq + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.provider, &other.provider) && self.content == other.content
    }
}

#[component]
pub fn AdminShell<R>(props: AdminShellProps<R>) -> Element
where
    R: Routable + Clone + PartialEq + 'static,
{
    let route = use_route::<R>();
    let shell = props.provider.shell(&route);
    let topbar = shell.topbar;
    let sections = shell.menu;

    rsx! {
        AdminWorkbench {
            topbar: rsx!(
                ThinTopbar {
                    brand: topbar.brand,
                    eyebrow: topbar.eyebrow,
                    title: topbar.title,
                    left_actions: if topbar.left.is_empty() {
                        None
                    } else {
                        Some(rsx! {
                            for action in topbar.left.into_iter() {
                                ActionButton::<R> { action }
                            }
                        })
                    },
                    right_actions: if topbar.right.is_empty() {
                        None
                    } else {
                        Some(rsx! {
                            for action in topbar.right.into_iter() {
                                ActionButton::<R> { action }
                            }
                        })
                    },
                }
            ),
            left: rsx!(
                Sidebar { side: SidebarSide::Left,
                    for section in sections.into_iter() {
                        SectionView::<R> { section }
                    }
                }
            ),
            center: rsx!(MainContent { {props.content} }),
            right: shell.right_panel.map(|panel| {
                rsx!(
                    Sidebar { side: SidebarSide::Right,
                        {panel}
                    }
                )
            }),
        }
    }
}

#[component]
fn ActionButton<R>(action: AdminAction<R>) -> Element
where
    R: Routable + Clone + PartialEq + 'static,
{
    let nav = use_navigator();
    let cmd = action.cmd.clone();

    rsx! {
        WorkbenchButton {
            class: action.class,
            tone: action.tone,
            title: action.title,
            onclick: move |_| {
                if let Some(on_run) = cmd.on_run.as_ref() {
                    on_run();
                }
                if let Some(target) = cmd.to.clone() {
                    nav.replace(target);
                }
            },
            {render_icon(action.icon)}
        }
    }
}

#[component]
fn SectionView<R>(section: AdminSection<R>) -> Element
where
    R: Routable + Clone + PartialEq + 'static,
{
    rsx! {
        SidebarSection { label: section.label,
            div { class: "nav-tree",
                for menu in section.menus.into_iter() {
                    MenuNode::<R> { menu, depth: 0 }
                }
            }
        }
    }
}

#[component]
fn MenuNode<R>(menu: AdminMenu<R>, depth: usize) -> Element
where
    R: Routable + Clone + PartialEq + 'static,
{
    let route = use_route::<R>();
    let nav = use_navigator();
    let is_active = (menu.is_active)(&route);
    let class = if is_active {
        if depth == 0 {
            "nav-item nav-item--active"
        } else {
            "nav-item nav-item--nested nav-item--active"
        }
    } else if depth == 0 {
        "nav-item"
    } else {
        "nav-item nav-item--nested"
    };
    let to = menu.to.clone();
    let on_select = menu.on_select.clone();
    let label = menu.label.clone();

    rsx! {
        div { class: "nav-tree__node",
            button {
                "type": "button",
                class: class,
                onclick: move |_| {
                    if let Some(on_select) = on_select.as_ref() {
                        on_select();
                    }
                    if let Some(target) = to.clone() {
                        nav.push(target);
                    }
                },
                span { class: "nav-item__main", "{label}" }
            }
            if !menu.children.is_empty() {
                div { class: "nav-tree__children",
                    for child in menu.children.into_iter() {
                        MenuNode::<R> { menu: child, depth: depth + 1 }
                    }
                }
            }
        }
    }
}

fn render_icon(icon: AdminActionIcon) -> Element {
    match icon {
        AdminActionIcon::Bell => rsx!(Icon {
            width: 16,
            height: 16,
            icon: LdBell
        }),
        AdminActionIcon::LogOut => rsx!(Icon {
            width: 16,
            height: 16,
            icon: LdLogOut
        }),
        AdminActionIcon::Moon => rsx!(Icon {
            width: 16,
            height: 16,
            icon: LdMoonStar
        }),
        AdminActionIcon::Search => rsx!(Icon {
            width: 16,
            height: 16,
            icon: LdSearch
        }),
        AdminActionIcon::Sun => rsx!(Icon {
            width: 16,
            height: 16,
            icon: LdSun
        }),
    }
}
