use az_dioxus_components::az_card::AzCard;
use az_dioxus_components::az_table::{
    AzTable, AzTableBody, AzTableCaption, AzTableCell, AzTableFooter, AzTableHead,
    AzTableHeaderCell, AzTableRow,
};
use dioxus::prelude::*;

#[test]
fn public_components_compile_in_a_single_composition() {
    // 验证公开模块足以组合出完整表格界面，而不需要依赖私有路径。
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
