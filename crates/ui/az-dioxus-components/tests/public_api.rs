use az_dioxus_components::grammar_search::{GrammarSearchField, parse_grammar_search_query};
use az_dioxus_components::neobrutal::{
    Badge, Card, ContentSlot, HeaderBar, Hero, IconButton, ModelButton, NavLink, Page, PluginGroup,
    Shell, Sidebar, SidebarToggle, TitlebarControls, TitlebarNav, Workspace, WorkspaceBody,
};
use az_dioxus_components::surface_card::SurfaceCard;
use az_dioxus_components::table::{
    Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeaderCell, TableRow,
};
use dioxus::prelude::*;

#[test]
fn public_components_compile_in_a_single_composition() {
    // 验证公开模块足以组合出完整表格界面，而不需要依赖私有路径。
    let field = GrammarSearchField::new("tag", "标签");
    let parsed_query = parse_grammar_search_query("keyword:edge; tag:runtime");
    assert_eq!(field.key, "tag");
    assert_eq!(parsed_query.values_for("tag"), vec!["runtime"]);

    let markup = dioxus_ssr::render_element(rsx! {
        SurfaceCard {
            Table {
                TableCaption { "Nodes" }
                TableHead {
                    TableRow {
                        TableHeaderCell { "Name" }
                        TableHeaderCell { "Status" }
                    }
                }
                TableBody {
                    TableRow {
                        TableCell { "edge-01" }
                        TableCell { "healthy" }
                    }
                }
                TableFooter {
                    TableRow {
                        TableCell { "1 total" }
                        TableCell { "ok" }
                    }
                }
            }
        }
    });

    assert!(markup.contains("surface-card"));
}

#[test]
fn neobrutal_components_render_stable_ssr_classes() {
    let markup = dioxus_ssr::render_element(rsx! {
        Page {
            Hero { compact: true,
                "Hero"
            }
            Card { accent: true, selected: true,
                Badge { accent: true, "Ready" }
            }
        }
    });

    assert!(markup.contains("page"));
    assert!(markup.contains("hero--compact"));
    assert!(markup.contains("card--accent"));
    assert!(markup.contains("card--selected"));
    assert!(markup.contains("badge--accent"));
}

#[test]
fn neobrutal_shell_components_render_stable_ssr_classes() {
    let markup = dioxus_ssr::render_element(rsx! {
        Shell { collapsed: true,
            TitlebarControls {
                SidebarToggle { expanded: false }
                TitlebarNav { label: "‹" }
                TitlebarNav { label: "›", disabled: true }
            }
            Sidebar {
                PluginGroup {
                    NavLink {
                        href: "/?route=/algorithms",
                        icon: "◆",
                        label: "算法",
                        detail: "10",
                        active: true,
                        plugin: true,
                    }
                }
            }
            Workspace {
                HeaderBar {
                    ModelButton {}
                    IconButton { id: "theme-toggle", href: "#", aria_label: "切换主题", "◐" }
                }
                WorkspaceBody { lowcode: true,
                    ContentSlot { plugin: true, "Plugin body" }
                }
            }
        }
    });

    assert!(markup.contains("shell"));
    assert!(markup.contains("shell--collapsed"));
    assert!(markup.contains("titlebar-controls"));
    assert!(markup.contains("sidebar-toggle"));
    assert!(markup.contains("titlebar-nav--disabled"));
    assert!(markup.contains("sidebar"));
    assert!(markup.contains("plugin-group"));
    assert!(markup.contains("nav-button--active"));
    assert!(markup.contains("workspace"));
    assert!(markup.contains("header-bar"));
    assert!(markup.contains("model-button"));
    assert!(markup.contains("icon-button"));
    assert!(markup.contains("workspace__body--lowcode"));
    assert!(markup.contains("content-center-slot--plugin"));
}
