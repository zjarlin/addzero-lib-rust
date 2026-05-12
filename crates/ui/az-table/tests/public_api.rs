#![allow(non_snake_case)]

use az_table::{AzTable, AzTableAlign, AzTableCell, AzTableColumn, AzTableRow};
use dioxus::prelude::*;

fn demo_columns() -> Vec<AzTableColumn> {
    vec![
        AzTableColumn {
            key: "name".into(),
            header: "Name".into(),
            class: None,
            align: AzTableAlign::Start,
        },
        AzTableColumn {
            key: "role".into(),
            header: "Role".into(),
            class: Some("col-role".into()),
            align: AzTableAlign::Center,
        },
    ]
}

fn demo_rows() -> Vec<AzTableRow> {
    vec![
        AzTableRow {
            key: "ada".into(),
            cells: vec![
                AzTableCell::from("Ada"),
                AzTableCell {
                    value: "Admin".into(),
                    class: Some("cell-role".into()),
                    align: Some(AzTableAlign::End),
                },
            ],
            class: Some("row-user".into()),
        },
        AzTableRow {
            key: "grace".into(),
            cells: vec!["Grace".into()],
            class: None,
        },
    ]
}

#[test]
fn az_table_should_render_caption_headers_and_rows() {
    let html = dioxus_ssr::render_element(rsx! {
        AzTable {
            columns: demo_columns(),
            rows: demo_rows(),
            caption: Some("Team".to_string()),
            class: "admin-grid",
            striped: true,
            bordered: true,
        }
    });

    assert!(html.contains(r#"<caption class="az-table__caption">Team</caption>"#));
    assert!(html.contains(r#"class="az-table az-table--striped az-table--bordered admin-grid""#));
    assert!(html.contains(
        r#"<th class="az-table__header-cell az-table__cell--start" scope="col">Name</th>"#
    ));
    assert!(html.contains(
        r#"<th class="az-table__header-cell az-table__cell--center col-role" scope="col">Role</th>"#
    ));
    assert!(html.contains(r#"<tr class="az-table__row row-user"><td class="az-table__cell az-table__cell--start">Ada</td><td class="az-table__cell az-table__cell--end cell-role">Admin</td></tr>"#));
}

#[test]
fn az_table_should_pad_short_rows_to_match_column_count() {
    let html = dioxus_ssr::render_element(rsx! {
        AzTable {
            columns: demo_columns(),
            rows: demo_rows(),
        }
    });

    assert!(html.contains(r#"<tr class="az-table__row"><td class="az-table__cell az-table__cell--start">Grace</td><td class="az-table__cell az-table__cell--center"></td></tr>"#));
}

#[test]
fn az_table_should_render_empty_state_when_rows_missing() {
    let html = dioxus_ssr::render_element(rsx! {
        AzTable {
            columns: demo_columns(),
            empty_label: "Nothing here".to_string(),
            dense: true,
        }
    });

    assert!(html.contains(r#"class="az-table az-table--dense""#));
    assert!(html.contains(r#"<tr class="az-table__row az-table__row--empty"><td class="az-table__cell az-table__cell--empty" colspan="2">Nothing here</td></tr>"#));
}
