#![allow(non_snake_case)]

use az_dioxus_components::az_data_table::{
    AzDataTable, AzDataTableAlign, AzDataTableCell, AzDataTableColumn, AzDataTableRow,
};
use dioxus::prelude::*;

fn demo_columns() -> Vec<AzDataTableColumn> {
    vec![
        AzDataTableColumn {
            key: "name".into(),
            header: "Name".into(),
            class: None,
            align: AzDataTableAlign::Start,
            frozen: true,
            sticky_left: Some("0px".into()),
        },
        AzDataTableColumn {
            key: "role".into(),
            header: "Role".into(),
            class: Some("col-role".into()),
            align: AzDataTableAlign::Center,
            frozen: false,
            sticky_left: None,
        },
    ]
}

fn demo_rows() -> Vec<AzDataTableRow> {
    vec![
        AzDataTableRow {
            key: "ada".into(),
            cells: vec![
                AzDataTableCell::from("Ada"),
                AzDataTableCell {
                    value: "Admin".into(),
                    class: Some("cell-role".into()),
                    align: Some(AzDataTableAlign::End),
                },
            ],
            class: Some("row-user".into()),
        },
        AzDataTableRow {
            key: "grace".into(),
            cells: vec!["Grace".into()],
            class: None,
        },
    ]
}

#[test]
fn az_data_table_should_render_caption_headers_and_rows() {
    let html = dioxus_ssr::render_element(rsx! {
        AzDataTable {
            columns: demo_columns(),
            rows: demo_rows(),
            caption: Some("Team".to_string()),
            class: "admin-grid",
            striped: true,
            bordered: true,
        }
    });

    assert!(html.contains(r#"<caption class="az-table__caption">Team</caption>"#));
    assert!(html.contains(
        r#"class="az-table az-table--striped az-table--bordered az-table--frozen-header admin-grid""#
    ));
    assert!(html.contains(
        r#"<th class="az-table__header-cell az-table__cell--start az-table__cell--frozen" style="left:0px;" scope="col">Name</th>"#
    ));
    assert!(html.contains(
        r#"<th class="az-table__header-cell az-table__cell--center col-role" scope="col">Role</th>"#
    ));
    assert!(html.contains(r#"<tr class="az-table__row row-user"><td class="az-table__cell az-table__cell--start az-table__cell--frozen" style="left:0px;">Ada</td><td class="az-table__cell az-table__cell--end cell-role">Admin</td></tr>"#));
    assert!(!html.contains(r#"style="""#));
}

#[test]
fn az_data_table_should_pad_short_rows_to_match_column_count() {
    let html = dioxus_ssr::render_element(rsx! {
        AzDataTable {
            columns: demo_columns(),
            rows: demo_rows(),
        }
    });

    assert!(html.contains(r#"<tr class="az-table__row"><td class="az-table__cell az-table__cell--start az-table__cell--frozen" style="left:0px;">Grace</td><td class="az-table__cell az-table__cell--center"></td></tr>"#));
    assert!(!html.contains(r#"style="""#));
}

#[test]
fn az_data_table_should_render_empty_state_when_rows_missing() {
    let html = dioxus_ssr::render_element(rsx! {
        AzDataTable {
            columns: demo_columns(),
            empty_label: "Nothing here".to_string(),
            dense: true,
        }
    });

    assert!(html.contains(r#"class="az-table az-table--dense az-table--frozen-header""#));
    assert!(html.contains(r#"<tr class="az-table__row az-table__row--empty"><td class="az-table__cell az-table__cell--empty" colspan="2">Nothing here</td></tr>"#));
    assert!(!html.contains(r#"style="""#));
}
