#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

automod::dir!(pub "src");

/// 组合公开 UI 基础组件时使用的便捷导出。
pub mod prelude {
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
