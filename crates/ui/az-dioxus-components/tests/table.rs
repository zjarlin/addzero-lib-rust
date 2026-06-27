use az_dioxus_components::table::{
    Table, TableBody, TableCaption, TableCell, TableHead, TableHeaderCell, TableRow,
};
use dioxus::prelude::*;

#[test]
fn table_renders_variant_classes_on_the_table_root() {
    let markup = dioxus_ssr::render_element(rsx! {
        Table {
            class: "operations-grid",
            dense: true,
            striped: true,
            bordered: true,
            TableBody {
                TableRow {
                    TableCell { "edge-01" }
                }
            }
        }
    });

    assert!(markup.contains(r#"data-az-style="az-dioxus-components""#));
    assert!(markup.contains("table-view table-view--dense table-view--striped table-view--bordered table-view--frozen-header operations-grid"));
    assert!(markup.contains(r#"<td class="table-view__cell">edge-01</td>"#));
}

#[test]
fn table_supports_semantic_composition_for_caption_head_and_numeric_cells() {
    let markup = dioxus_ssr::render_element(rsx! {
        Table {
            TableCaption { "Runtime nodes" }
            TableHead {
                TableRow { selected: true, style: "background:red;",
                    TableHeaderCell { "Name" }
                    TableHeaderCell { numeric: true, "Latency" }
                }
            }
            TableBody {
                TableRow {
                    TableCell { "edge-01" }
                    TableCell { numeric: true, "42ms" }
                }
            }
        }
    });

    assert!(markup.contains(r#"<caption class="table-view__caption">Runtime nodes</caption>"#));
    assert!(markup.contains(
        r#"<tr class="table-view__row table-view__row--selected" style="background:red;">"#
    ));
    assert!(markup.contains(
        r#"<th class="table-view__header-cell table-view__cell--numeric" scope="col">Latency</th>"#
    ));
    assert!(markup.contains(r#"<td class="table-view__cell table-view__cell--numeric">42ms</td>"#));
}

#[test]
fn table_cell_can_span_multiple_columns() {
    let markup = dioxus_ssr::render_element(rsx! {
        Table {
            TableBody {
                TableRow {
                    TableCell { colspan: 2, "Nothing here" }
                }
            }
        }
    });

    assert!(markup.contains(r#"<td class="table-view__cell" colspan="2">Nothing here</td>"#));
}
