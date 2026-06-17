#![allow(non_snake_case)]

use az_dioxus_components::data_table::{
    DataTable, DataTableAlign, DataTableCell, DataTableColumn, DataTableRow,
};
use dioxus::prelude::*;

fn demo_columns() -> Vec<DataTableColumn> {
    vec![
        DataTableColumn {
            key: "name".into(),
            header: "Name".into(),
            class: None,
            align: DataTableAlign::Start,
            frozen: true,
            sticky_left: Some("0px".into()),
        },
        DataTableColumn {
            key: "role".into(),
            header: "Role".into(),
            class: Some("col-role".into()),
            align: DataTableAlign::Center,
            frozen: false,
            sticky_left: None,
        },
    ]
}

fn demo_rows() -> Vec<DataTableRow> {
    vec![
        DataTableRow {
            key: "ada".into(),
            cells: vec![
                DataTableCell::from("Ada"),
                DataTableCell {
                    value: "Admin".into(),
                    class: Some("cell-role".into()),
                    align: Some(DataTableAlign::End),
                },
            ],
            class: Some("row-user".into()),
        },
        DataTableRow {
            key: "grace".into(),
            cells: vec!["Grace".into()],
            class: None,
        },
    ]
}

#[test]
fn data_table_should_render_caption_headers_and_rows() {
    let html = dioxus_ssr::render_element(rsx! {
        DataTable {
            columns: demo_columns(),
            rows: demo_rows(),
            caption: Some("Team".to_string()),
            class: "admin-grid",
            striped: true,
            bordered: true,
        }
    });

    assert!(html.contains(r#"<caption class="table-view__caption">Team</caption>"#));
    assert!(html.contains(
        r#"class="table-view table-view--striped table-view--bordered table-view--frozen-header admin-grid""#
    ));
    assert!(html.contains(
        r#"<th class="table-view__header-cell table-view__cell--start table-view__cell--frozen" style="left:0px;" scope="col">Name</th>"#
    ));
    assert!(html.contains(
        r#"<th class="table-view__header-cell table-view__cell--center col-role" scope="col">Role</th>"#
    ));
    assert!(html.contains(r#"<tr class="table-view__row row-user"><td class="table-view__cell table-view__cell--start table-view__cell--frozen" style="left:0px;">Ada</td><td class="table-view__cell table-view__cell--end cell-role">Admin</td></tr>"#));
    assert!(!html.contains(r#"style="""#));
}

#[test]
fn data_table_should_pad_short_rows_to_match_column_count() {
    let html = dioxus_ssr::render_element(rsx! {
        DataTable {
            columns: demo_columns(),
            rows: demo_rows(),
        }
    });

    assert!(html.contains(r#"<tr class="table-view__row"><td class="table-view__cell table-view__cell--start table-view__cell--frozen" style="left:0px;">Grace</td><td class="table-view__cell table-view__cell--center"></td></tr>"#));
    assert!(!html.contains(r#"style="""#));
}

#[test]
fn data_table_should_render_empty_state_when_rows_missing() {
    let html = dioxus_ssr::render_element(rsx! {
        DataTable {
            columns: demo_columns(),
            empty_label: "Nothing here".to_string(),
            dense: true,
        }
    });

    assert!(html.contains(r#"class="table-view table-view--dense table-view--frozen-header""#));
    assert!(html.contains(r#"<tr class="table-view__row table-view__row--empty"><td class="table-view__cell table-view__cell--empty" colspan="2">Nothing here</td></tr>"#));
    assert!(!html.contains(r#"style="""#));
}
