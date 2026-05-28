#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod class_name;

/// 遵循 `az-card` class 契约的卡片基础组件。
pub mod az_card;
/// 遵循 `az-table` class 契约的表格基础组件。
pub mod az_table;

/// 组合公开 UI 基础组件时使用的便捷导出。
pub mod prelude {
    //! 本 crate 公开 Dioxus 组件的便捷导出。

    pub use crate::az_card::AzCard;
    pub use crate::az_table::{
        AzTable, AzTableBody, AzTableCaption, AzTableCell, AzTableFooter, AzTableHead,
        AzTableHeaderCell, AzTableRow,
    };
}
