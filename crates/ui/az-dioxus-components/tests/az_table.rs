use az_dioxus_components::az_table::{
    AzTable, AzTableBody, AzTableCaption, AzTableCell, AzTableHead, AzTableHeaderCell, AzTableRow,
};
use dioxus::prelude::*;

#[test]
fn az_table_renders_variant_classes_on_the_table_root() {
    let markup = dioxus_ssr::render_element(rsx! {
        AzTable {
            class: "operations-grid",
            dense: true,
            striped: true,
            bordered: true,
            AzTableBody {
                AzTableRow {
                    AzTableCell { "edge-01" }
                }
            }
        }
    });

    assert_eq!(
        markup,
        "<div class=\"az-table__scroller\"><table class=\"az-table az-table--dense az-table--striped az-table--bordered az-table--frozen-header operations-grid\"><tbody class=\"az-table__body\"><tr class=\"az-table__row\"><td class=\"az-table__cell\">edge-01</td></tr></tbody></table></div>"
    );
}

#[test]
fn az_table_supports_semantic_composition_for_caption_head_and_numeric_cells() {
    let markup = dioxus_ssr::render_element(rsx! {
        AzTable {
            AzTableCaption { "Runtime nodes" }
            AzTableHead {
                AzTableRow { selected: true, style: "background:red;",
                    AzTableHeaderCell { "Name" }
                    AzTableHeaderCell { numeric: true, "Latency" }
                }
            }
            AzTableBody {
                AzTableRow {
                    AzTableCell { "edge-01" }
                    AzTableCell { numeric: true, "42ms" }
                }
            }
        }
    });

    assert_eq!(
        markup,
        "<div class=\"az-table__scroller\"><table class=\"az-table az-table--frozen-header\"><caption class=\"az-table__caption\">Runtime nodes</caption><thead class=\"az-table__head\"><tr class=\"az-table__row az-table__row--selected\" style=\"background:red;\"><th class=\"az-table__header-cell\" scope=\"col\">Name</th><th class=\"az-table__header-cell az-table__cell--numeric\" scope=\"col\">Latency</th></tr></thead><tbody class=\"az-table__body\"><tr class=\"az-table__row\"><td class=\"az-table__cell\">edge-01</td><td class=\"az-table__cell az-table__cell--numeric\">42ms</td></tr></tbody></table></div>"
    );
}

#[test]
fn az_table_cell_can_span_multiple_columns() {
    let markup = dioxus_ssr::render_element(rsx! {
        AzTable {
            AzTableBody {
                AzTableRow {
                    AzTableCell { colspan: 2, "Nothing here" }
                }
            }
        }
    });

    assert_eq!(
        markup,
        "<div class=\"az-table__scroller\"><table class=\"az-table az-table--frozen-header\"><tbody class=\"az-table__body\"><tr class=\"az-table__row\"><td class=\"az-table__cell\" colspan=\"2\">Nothing here</td></tr></tbody></table></div>"
    );
}
