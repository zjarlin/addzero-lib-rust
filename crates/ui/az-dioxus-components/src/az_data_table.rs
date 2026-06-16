//! 基于结构化列和行数据渲染的表格组件。

#![allow(non_snake_case)]

use az_derive_aliases::{
    apply, dioxus_props, impl_from_with_default, plain_default_copy_eq, plain_default_eq, plain_eq,
};
use dioxus::prelude::*;

use crate::az_table::{
    AzTable as PrimitiveAzTable, AzTableBody, AzTableCaption, AzTableCell as PrimitiveAzTableCell,
    AzTableHead, AzTableHeaderCell, AzTableRow as PrimitiveAzTableRow,
};

/// 表头和单元格内容的水平对齐方式。
#[apply(plain_default_copy_eq)]
pub enum AzDataTableAlign {
    /// 内容左对齐。
    #[default]
    Start,
    /// 内容居中对齐。
    Center,
    /// 内容右对齐。
    End,
}

impl AzDataTableAlign {
    fn class_name(self) -> &'static str {
        match self {
            Self::Start => "az-table__cell--start",
            Self::Center => "az-table__cell--center",
            Self::End => "az-table__cell--end",
        }
    }
}
/// [`AzDataTable`] 的列定义。
#[apply(plain_eq)]
pub struct AzDataTableColumn {
    /// 调用方使用的稳定列键。
    pub key: String,
    /// 表头展示文本。
    pub header: String,
    /// 追加到渲染后表头单元格上的可选 class。
    pub class: Option<String>,
    /// 表头对齐方式。
    pub align: AzDataTableAlign,
}

/// [`AzDataTable`] 行内的单元格数据。
#[apply(plain_default_eq)]
pub struct AzDataTableCell {
    /// 可见文本内容。
    pub value: String,
    /// 追加到渲染后数据单元格上的可选 class。
    pub class: Option<String>,
    /// 单元格级别的对齐覆盖。
    pub align: Option<AzDataTableAlign>,
}

impl_from_with_default!(&str => AzDataTableCell {
    value: |source| source.to_owned(),
});

impl_from_with_default!(String => AzDataTableCell {
    value: |source| source,
});

/// [`AzDataTable`] 的行数据。
#[apply(plain_eq)]
pub struct AzDataTableRow {
    /// 调用方使用的稳定行键。
    pub key: String,
    /// 该行要渲染的单元格。
    pub cells: Vec<AzDataTableCell>,
    /// 追加到渲染后行元素上的可选 class。
    pub class: Option<String>,
}

/// [`AzDataTable`] 的组件属性。
#[apply(dioxus_props)]
pub struct AzDataTableProps {
    /// 列定义集合。
    pub columns: Vec<AzDataTableColumn>,
    /// 表格行集合。
    #[props(default)]
    pub rows: Vec<AzDataTableRow>,
    /// 渲染到 `<caption>` 内的可选标题。
    #[props(default)]
    pub caption: Option<String>,
    /// 追加到根 `<table>` 上的可选自定义 class。
    #[props(default, into)]
    pub class: String,
    /// `rows` 为空时渲染的空状态文案。
    #[props(default = String::from("No data"))]
    pub empty_label: String,
    /// 添加 `az-table--striped` 修饰 class。
    #[props(default = false)]
    pub striped: bool,
    /// 添加 `az-table--bordered` 修饰 class。
    #[props(default = false)]
    pub bordered: bool,
    /// 添加 `az-table--dense` 修饰 class。
    #[props(default = false)]
    pub dense: bool,
}

/// 根据结构化列和行数据渲染完整表格。
pub fn AzDataTable(props: AzDataTableProps) -> Element {
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

fn effective_column_count(columns: &[AzDataTableColumn], rows: &[AzDataTableRow]) -> usize {
    if !columns.is_empty() {
        return columns.len();
    }

    rows.iter()
        .map(|row| row.cells.len())
        .max()
        .unwrap_or(1)
        .max(1)
}

fn normalize_cells(row: &AzDataTableRow, column_count: usize) -> Vec<AzDataTableCell> {
    let mut cells = row.cells.clone();
    cells.resize(column_count, AzDataTableCell::default());
    cells.truncate(column_count);
    cells
}

fn build_header_class(column: &AzDataTableColumn) -> String {
    join_classes([
        column.align.class_name(),
        column.class.as_deref().unwrap_or_default(),
    ])
}

fn build_cell_class(user_class: Option<&str>, align: Option<AzDataTableAlign>) -> String {
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
        let row = AzDataTableRow {
            key: "grace".into(),
            cells: vec!["Grace".into()],
            class: None,
        };

        let normalized = normalize_cells(&row, 2);

        // 即使某一行缺少值，表格也必须保持稳定的列几何结构。
        assert_eq!(
            normalized,
            vec![AzDataTableCell::from("Grace"), AzDataTableCell::default()]
        );
    }

    #[test]
    fn effective_column_count_prefers_declared_columns() {
        let columns = vec![
            AzDataTableColumn {
                key: "name".into(),
                header: "Name".into(),
                class: None,
                align: AzDataTableAlign::Start,
            },
            AzDataTableColumn {
                key: "role".into(),
                header: "Role".into(),
                class: None,
                align: AzDataTableAlign::Center,
            },
        ];
        let rows = vec![AzDataTableRow {
            key: "ada".into(),
            cells: vec!["Ada".into()],
            class: None,
        }];

        // 显式声明的列是空状态 colspan 和行补齐逻辑的事实来源。
        assert_eq!(effective_column_count(&columns, &rows), 2);
    }
}
