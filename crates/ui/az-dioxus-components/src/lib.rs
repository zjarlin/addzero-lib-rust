#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod class_name;

/// 遵循 `az-card` class 契约的卡片基础组件。
pub mod az_card;
/// 基于结构化列和行数据渲染的表格组件。
pub mod az_data_table;
/// GitHub/JQL 风格的语法式搜索输入组件和解析模型。
pub mod az_grammar_search;
/// 遵循 `az-table` class 契约的表格基础组件。
pub mod az_table;

/// 组合公开 UI 基础组件时使用的便捷导出。
pub mod prelude {
    //! 本 crate 公开 Dioxus 组件的便捷导出。

    pub use crate::az_card::AzCard;
    pub use crate::az_data_table::{
        AzDataTable, AzDataTableAlign, AzDataTableCell, AzDataTableColumn, AzDataTableRow,
    };
    pub use crate::az_grammar_search::{
        AzGrammarSearchField, AzGrammarSearchInput, GrammarSearchFilter, GrammarSearchQuery,
        GrammarSearchTerm, parse_grammar_search_query,
    };
    pub use crate::az_table::{
        AzTable, AzTableBody, AzTableCaption, AzTableCell, AzTableFooter, AzTableHead,
        AzTableHeaderCell, AzTableRow,
    };
}
