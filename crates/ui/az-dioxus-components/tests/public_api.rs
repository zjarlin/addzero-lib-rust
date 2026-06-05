use az_dioxus_components::az_card::AzCard;
use az_dioxus_components::az_grammar_search::{AzGrammarSearchField, parse_grammar_search_query};
use az_dioxus_components::az_table::{
    AzTable, AzTableBody, AzTableCaption, AzTableCell, AzTableFooter, AzTableHead,
    AzTableHeaderCell, AzTableRow,
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
