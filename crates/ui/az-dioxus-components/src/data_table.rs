//! 基于结构化列和行数据渲染的表格组件。

#![allow(non_snake_case)]

use az_derive_aliases::{
    apply, dioxus_props, impl_from_with_default, plain_default_copy_eq, plain_default_eq, plain_eq,
};
use dioxus::prelude::*;

use crate::table::{
    Table as PrimitiveTable, TableBody, TableCaption, TableCell as PrimitiveTableCell, TableHead,
    TableHeaderCell, TableRow as PrimitiveTableRow,
};

/// 表头和单元格内容的水平对齐方式。
#[apply(plain_default_copy_eq)]
pub enum DataTableAlign {
    /// 内容左对齐。
    #[default]
    Start,
    /// 内容居中对齐。
    Center,
    /// 内容右对齐。
    End,
}

impl DataTableAlign {
    fn class_name(self) -> &'static str {
        match self {
            Self::Start => "table-view__cell--start",
            Self::Center => "table-view__cell--center",
            Self::End => "table-view__cell--end",
        }
    }
}
/// [`DataTable`] 的列定义。
#[apply(plain_eq)]
pub struct DataTableColumn {
    /// 调用方使用的稳定列键。
    pub key: String,
    /// 表头展示文本。
    pub header: String,
    /// 追加到渲染后表头单元格上的可选 class。
    pub class: Option<String>,
    /// 表头对齐方式。
    pub align: DataTableAlign,
    /// 开启后该列会固定在水平滚动区域左侧。
    pub frozen: bool,
    /// 固定列的左侧偏移，支持 `px` 或 `rem` 等 CSS 长度。
    pub sticky_left: Option<String>,
}

/// [`DataTable`] 行内的单元格数据。
#[apply(plain_default_eq)]
pub struct DataTableCell {
    /// 可见文本内容。
    pub value: String,
    /// 追加到渲染后数据单元格上的可选 class。
    pub class: Option<String>,
    /// 单元格级别的对齐覆盖。
    pub align: Option<DataTableAlign>,
}

impl_from_with_default!(&str => DataTableCell {
    value: |source| source.to_owned(),
});

impl_from_with_default!(String => DataTableCell {
    value: |source| source,
});

/// [`DataTable`] 的行数据。
#[apply(plain_eq)]
pub struct DataTableRow {
    /// 调用方使用的稳定行键。
    pub key: String,
    /// 该行要渲染的单元格。
    pub cells: Vec<DataTableCell>,
    /// 追加到渲染后行元素上的可选 class。
    pub class: Option<String>,
}

/// [`DataTable`] 的组件属性。
#[apply(dioxus_props)]
pub struct DataTableProps {
    /// 列定义集合。
    pub columns: Vec<DataTableColumn>,
    /// 表格行集合。
    #[props(default)]
    pub rows: Vec<DataTableRow>,
    /// 渲染到 `<caption>` 内的可选标题。
    #[props(default)]
    pub caption: Option<String>,
    /// 追加到根 `<table>` 上的可选自定义 class。
    #[props(default, into)]
    pub class: String,
    /// `rows` 为空时渲染的空状态文案。
    #[props(default = String::from("No data"))]
    pub empty_label: String,
    /// 添加 `table-view--striped` 修饰 class。
    #[props(default = false)]
    pub striped: bool,
    /// 添加 `table-view--bordered` 修饰 class。
    #[props(default = false)]
    pub bordered: bool,
    /// 添加 `table-view--dense` 修饰 class。
    #[props(default = false)]
    pub dense: bool,
    /// 表头是否固定在滚动容器顶部。
    #[props(default = true)]
    pub frozen_header: bool,
}

/// 根据结构化列和行数据渲染完整表格。
pub fn DataTable(props: DataTableProps) -> Element {
    let column_count = effective_column_count(&props.columns, &props.rows);

    rsx! {
        PrimitiveTable {
            class: props.class,
            dense: props.dense,
            striped: props.striped,
            bordered: props.bordered,
            frozen_header: props.frozen_header,
            if let Some(caption) = props.caption.as_deref() {
                TableCaption { "{caption}" }
            }
            if !props.columns.is_empty() {
                TableHead {
                    PrimitiveTableRow {
                        {props.columns.iter().map(|column| {
                            let class = build_header_class(column);
                            let header = column.header.clone();
                            rsx! {
                                TableHeaderCell {
                                    class: class,
                                    style: sticky_style(column),
                                    "{header}"
                                }
                            }
                        })}
                    }
                }
            }
            TableBody {
                if props.rows.is_empty() {
                    PrimitiveTableRow { class: "table-view__row--empty",
                        PrimitiveTableCell {
                            class: "table-view__cell--empty",
                            colspan: column_count,
                            "{props.empty_label}"
                        }
                    }
                } else {
                    {props.rows.iter().map(|row| {
                        let row_class = row.class.clone().unwrap_or_default();
                        let normalized = normalize_cells(row, column_count);
                        rsx! {
                            PrimitiveTableRow { class: row_class,
                                {normalized.into_iter().enumerate().map(|(index, cell)| {
                                    let column = props.columns.get(index);
                                    let fallback_align = column.map(|column| column.align);
                                    let class = build_cell_class(
                                        cell.class.as_deref(),
                                        cell.align.or(fallback_align),
                                        column.map(|column| column.frozen).unwrap_or_default(),
                                    );
                                    let value = cell.value;
                                    rsx! {
                                        PrimitiveTableCell {
                                            class: class,
                                            style: sticky_style_for_index(&props.columns, index),
                                            "{value}"
                                        }
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

fn effective_column_count(columns: &[DataTableColumn], rows: &[DataTableRow]) -> usize {
    if !columns.is_empty() {
        return columns.len();
    }

    rows.iter()
        .map(|row| row.cells.len())
        .max()
        .unwrap_or(1)
        .max(1)
}

fn normalize_cells(row: &DataTableRow, column_count: usize) -> Vec<DataTableCell> {
    let mut cells = row.cells.clone();
    cells.resize(column_count, DataTableCell::default());
    cells.truncate(column_count);
    cells
}

fn build_header_class(column: &DataTableColumn) -> String {
    join_classes([
        column.align.class_name(),
        frozen_column_class(column.frozen),
        column.class.as_deref().unwrap_or_default(),
    ])
}

fn build_cell_class(
    user_class: Option<&str>,
    align: Option<DataTableAlign>,
    frozen: bool,
) -> String {
    join_classes([
        align.unwrap_or_default().class_name(),
        frozen_column_class(frozen),
        user_class.unwrap_or_default(),
    ])
}

fn frozen_column_class(frozen: bool) -> &'static str {
    if frozen {
        "table-view__cell--frozen"
    } else {
        ""
    }
}

fn sticky_style(column: &DataTableColumn) -> String {
    if column.frozen {
        format!("left:{};", column.sticky_left.as_deref().unwrap_or("0px"))
    } else {
        String::new()
    }
}

fn sticky_style_for_index(columns: &[DataTableColumn], index: usize) -> String {
    columns.get(index).map(sticky_style).unwrap_or_default()
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
        let row = DataTableRow {
            key: "grace".into(),
            cells: vec!["Grace".into()],
            class: None,
        };

        let normalized = normalize_cells(&row, 2);

        // 即使某一行缺少值，表格也必须保持稳定的列几何结构。
        assert_eq!(
            normalized,
            vec![DataTableCell::from("Grace"), DataTableCell::default()]
        );
    }

    #[test]
    fn effective_column_count_prefers_declared_columns() {
        let columns = vec![
            DataTableColumn {
                key: "name".into(),
                header: "Name".into(),
                class: None,
                align: DataTableAlign::Start,
                frozen: false,
                sticky_left: None,
            },
            DataTableColumn {
                key: "role".into(),
                header: "Role".into(),
                class: None,
                align: DataTableAlign::Center,
                frozen: false,
                sticky_left: None,
            },
        ];
        let rows = vec![DataTableRow {
            key: "ada".into(),
            cells: vec!["Ada".into()],
            class: None,
        }];

        // 显式声明的列是空状态 colspan 和行补齐逻辑的事实来源。
        assert_eq!(effective_column_count(&columns, &rows), 2);
    }
}
