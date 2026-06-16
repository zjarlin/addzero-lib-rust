use az_dioxus_components::az_card::AzCard;
use az_dioxus_components::az_grammar_search::{AzGrammarSearchField, parse_grammar_search_query};
use az_dioxus_components::az_table::{
    AzTable, AzTableBody, AzTableCaption, AzTableCell, AzTableFooter, AzTableHead,
    AzTableHeaderCell, AzTableRow,
};
use az_dioxus_components::neobrutal::{
    NbBadge, NbCard, NbContentSlot, NbHeaderBar, NbHero, NbIconButton, NbModelButton, NbNavLink,
    NbPage, NbPluginGroup, NbShell, NbSidebar, NbSidebarToggle, NbTitlebarControls, NbTitlebarNav,
    NbWorkspace, NbWorkspaceBody,
};
use dioxus::prelude::*;

#[test]
fn public_components_compile_in_a_single_composition() {
    // 验证公开模块足以组合出完整表格界面，而不需要依赖私有路径。
    let field = AzGrammarSearchField::new("tag", "标签");
    let parsed_query = parse_grammar_search_query("keyword:edge; tag:runtime");
    assert_eq!(field.key, "tag");
    assert_eq!(parsed_query.values_for("tag"), vec!["runtime"]);

    let markup = dioxus_ssr::render_element(rsx! {
        AzCard {
            AzTable {
                AzTableCaption { "Nodes" }
                AzTableHead {
                    AzTableRow {
                        AzTableHeaderCell { "Name" }
                        AzTableHeaderCell { "Status" }
                    }
                }
                AzTableBody {
                    AzTableRow {
                        AzTableCell { "edge-01" }
                        AzTableCell { "healthy" }
                    }
                }
                AzTableFooter {
                    AzTableRow {
                        AzTableCell { "1 total" }
                        AzTableCell { "ok" }
                    }
                }
            }
        }
    });

    assert!(markup.contains("az-card"));
}

#[test]
fn neobrutal_components_render_stable_ssr_classes() {
    let markup = dioxus_ssr::render_element(rsx! {
        NbPage {
            NbHero { compact: true,
                "Hero"
            }
            NbCard { accent: true, selected: true,
                NbBadge { accent: true, "Ready" }
            }
        }
    });

    assert!(markup.contains("nb-page"));
    assert!(markup.contains("nb-hero--compact"));
    assert!(markup.contains("nb-card--accent"));
    assert!(markup.contains("nb-card--selected"));
    assert!(markup.contains("nb-badge--accent"));
}

#[test]
fn neobrutal_shell_components_render_stable_ssr_classes() {
    let markup = dioxus_ssr::render_element(rsx! {
        NbShell { collapsed: true,
            NbTitlebarControls {
                NbSidebarToggle { expanded: false }
                NbTitlebarNav { label: "‹" }
                NbTitlebarNav { label: "›", disabled: true }
            }
            NbSidebar {
                NbPluginGroup {
                    NbNavLink {
                        href: "/?route=/algorithms",
                        icon: "◆",
                        label: "算法",
                        detail: "10",
                        active: true,
                        plugin: true,
                    }
                }
            }
            NbWorkspace {
                NbHeaderBar {
                    NbModelButton {}
                    NbIconButton { id: "theme-toggle", href: "#", aria_label: "切换主题", "◐" }
                }
                NbWorkspaceBody { lowcode: true,
                    NbContentSlot { plugin: true, "Plugin body" }
                }
            }
        }
    });

    assert!(markup.contains("nb-shell"));
    assert!(markup.contains("az-aio-shell--collapsed"));
    assert!(markup.contains("nb-titlebar-controls"));
    assert!(markup.contains("nb-sidebar-toggle"));
    assert!(markup.contains("nb-titlebar-nav--disabled"));
    assert!(markup.contains("nb-sidebar"));
    assert!(markup.contains("nb-plugin-group"));
    assert!(markup.contains("nb-nav-button--active"));
    assert!(markup.contains("nb-workspace"));
    assert!(markup.contains("nb-header-bar"));
    assert!(markup.contains("nb-model-button"));
    assert!(markup.contains("nb-icon-button"));
    assert!(markup.contains("nb-workspace-body--lowcode"));
    assert!(markup.contains("nb-content-slot--plugin"));
}
