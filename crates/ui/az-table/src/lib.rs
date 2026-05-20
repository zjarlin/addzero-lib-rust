#![allow(non_snake_case)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use az_dioxus_components::az_table::{
    AzTable as PrimitiveAzTable, AzTableBody, AzTableCaption, AzTableCell as PrimitiveAzTableCell,
    AzTableHead, AzTableHeaderCell, AzTableRow as PrimitiveAzTableRow,
};
use dioxus::prelude::*;

/// Horizontal alignment for headers and cells.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AzTableAlign {
    /// Left-aligned content.
    #[default]
    Start,
    /// Center-aligned content.
    Center,
    /// Right-aligned content.
    End,
}

impl AzTableAlign {
    fn class_name(self) -> &'static str {
        match self {
            Self::Start => "az-table__cell--start",
            Self::Center => "az-table__cell--center",
            Self::End => "az-table__cell--end",
        }
    }
}

/// Column definition for [`AzTable`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AzTableColumn {
    /// Stable column key used by the consumer.
    pub key: String,
    /// Header label text.
    pub header: String,
    /// Optional class appended to the rendered header cell.
    pub class: Option<String>,
    /// Header alignment.
    pub align: AzTableAlign,
}

/// Cell payload for a row in [`AzTable`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AzTableCell {
    /// Visible text content.
    pub value: String,
    /// Optional class appended to the rendered data cell.
    pub class: Option<String>,
    /// Cell-specific alignment override.
    pub align: Option<AzTableAlign>,
}

impl From<&str> for AzTableCell {
    fn from(value: &str) -> Self {
        Self {
            value: value.to_owned(),
            class: None,
            align: None,
        }
    }
}

impl From<String> for AzTableCell {
    fn from(value: String) -> Self {
        Self {
            value,
            class: None,
            align: None,
        }
    }
}

/// Row payload for [`AzTable`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AzTableRow {
    /// Stable row key used by the consumer.
    pub key: String,
    /// Rendered cells for the row.
    pub cells: Vec<AzTableCell>,
    /// Optional class appended to the rendered row.
    pub class: Option<String>,
}

/// Props for [`AzTable`].
#[derive(Props, Clone, PartialEq)]
pub struct AzTableProps {
    /// Column definitions.
    pub columns: Vec<AzTableColumn>,
    /// Table rows.
    #[props(default)]
    pub rows: Vec<AzTableRow>,
    /// Optional caption rendered inside `<caption>`.
    #[props(default)]
    pub caption: Option<String>,
    /// Optional custom class appended to the root `<table>`.
    #[props(default, into)]
    pub class: String,
    /// Empty-state copy rendered when `rows` is empty.
    #[props(default = String::from("No data"))]
    pub empty_label: String,
    /// Adds the `az-table--striped` modifier class.
    #[props(default = false)]
    pub striped: bool,
    /// Adds the `az-table--bordered` modifier class.
    #[props(default = false)]
    pub bordered: bool,
    /// Adds the `az-table--dense` modifier class.
    #[props(default = false)]
    pub dense: bool,
}

/// Renders a complete table from structured column and row data.
pub fn AzTable(props: AzTableProps) -> Element {
    let column_count = effective_column_count(&props.columns, &props.rows);

    rsx! {
        PrimitiveAzTable {
            class: props.class,
            dense: props.dense,
            striped: props.striped,
            bordered: props.bordered,
            if let Some(caption) = props.caption.as_deref() {
                AzTableCaption { "{caption}" }
            }
            if !props.columns.is_empty() {
                AzTableHead {
                    PrimitiveAzTableRow {
                        {props.columns.iter().map(|column| {
                            let class = build_header_class(column);
                            let header = column.header.clone();
                            rsx! {
                                AzTableHeaderCell {
                                    class: class,
                                    "{header}"
                                }
                            }
                        })}
                    }
                }
            }
            AzTableBody {
                if props.rows.is_empty() {
                    PrimitiveAzTableRow { class: "az-table__row--empty",
                        PrimitiveAzTableCell {
                            class: "az-table__cell--empty",
                            colspan: column_count,
                            "{props.empty_label}"
                        }
                    }
                } else {
                    {props.rows.iter().map(|row| {
                        let row_class = row.class.clone().unwrap_or_default();
                        let normalized = normalize_cells(row, column_count);
                        rsx! {
                            PrimitiveAzTableRow { class: row_class,
                                {normalized.into_iter().enumerate().map(|(index, cell)| {
                                    let fallback_align =
                                        props.columns.get(index).map(|column| column.align);
                                    let class = build_cell_class(
                                        cell.class.as_deref(),
                                        cell.align.or(fallback_align),
                                    );
                                    let value = cell.value;
                                    rsx! {
                                        PrimitiveAzTableCell { class: class, "{value}" }
                                    }
                                })}
                            }
                        }
                    })}
                }
            }
        }
    }
}

fn effective_column_count(columns: &[AzTableColumn], rows: &[AzTableRow]) -> usize {
    if !columns.is_empty() {
        return columns.len();
    }

    rows.iter()
        .map(|row| row.cells.len())
        .max()
        .unwrap_or(1)
        .max(1)
}

fn normalize_cells(row: &AzTableRow, column_count: usize) -> Vec<AzTableCell> {
    let mut cells = row.cells.clone();
    cells.resize(column_count, AzTableCell::default());
    cells.truncate(column_count);
    cells
}

fn build_header_class(column: &AzTableColumn) -> String {
    join_classes([
        column.align.class_name(),
        column.class.as_deref().unwrap_or_default(),
    ])
}

fn build_cell_class(user_class: Option<&str>, align: Option<AzTableAlign>) -> String {
    join_classes([
        align.unwrap_or_default().class_name(),
        user_class.unwrap_or_default(),
    ])
}

fn join_classes<const N: usize>(parts: [&str; N]) -> String {
    parts
        .into_iter()
        .filter(|part| !part.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_cells_pads_short_rows() {
        let row = AzTableRow {
            key: "grace".into(),
            cells: vec!["Grace".into()],
            class: None,
        };

        let normalized = normalize_cells(&row, 2);

        // The table must keep its column geometry stable even when a row has missing values.
        assert_eq!(
            normalized,
            vec![AzTableCell::from("Grace"), AzTableCell::default()]
        );
    }

    #[test]
    fn effective_column_count_prefers_declared_columns() {
        let columns = vec![
            AzTableColumn {
                key: "name".into(),
                header: "Name".into(),
                class: None,
                align: AzTableAlign::Start,
            },
            AzTableColumn {
                key: "role".into(),
                header: "Role".into(),
                class: None,
                align: AzTableAlign::Center,
            },
        ];
        let rows = vec![AzTableRow {
            key: "ada".into(),
            cells: vec!["Ada".into()],
            class: None,
        }];

        // Declared columns are the source of truth for empty-state colspan and row padding.
        assert_eq!(effective_column_count(&columns, &rows), 2);
    }
}
