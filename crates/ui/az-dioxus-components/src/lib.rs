#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

automod::dir!(pub "src");

/// 组合公开 UI 基础组件时使用的便捷导出。
pub mod prelude {
    pub use crate::az_accordion::AzAccordion;
    pub use crate::az_badge::{AzBadge, AzBadgeTone};
    pub use crate::az_button::{AzButton, AzButtonLink, AzButtonTone};
    pub use crate::az_card::AzCard;
    pub use crate::az_data_table::{
        AzDataTable, AzDataTableAlign, AzDataTableCell, AzDataTableColumn, AzDataTableRow,
    };
    pub use crate::az_form::{
        AzActionForm, AzCheckboxRow, AzFormGrid, AzFormRow, AzHiddenInput, AzInput, AzSelect,
        AzSelectOption,
    };
    pub use crate::az_grammar_search::{
        AzGrammarSearchField, AzGrammarSearchInput, GrammarSearchFilter, GrammarSearchQuery,
        GrammarSearchTerm, parse_grammar_search_query,
    };
    pub use crate::az_table::{
        AzTable, AzTableBody, AzTableCaption, AzTableCell, AzTableFooter, AzTableHead,
        AzTableHeaderCell, AzTableRow,
    };
    pub use crate::az_workbench::{
        AzPageHeader, AzSplitWorkbench, AzTableViewport, AzToolbar, AzWorkbenchDetail,
        AzWorkbenchDetailHeader, AzWorkbenchPage, AzWorkbenchTree, AzWorkbenchTreeHeader,
        AzWorkbenchTreeList,
    };
    pub use crate::neobrutal::{
        NbBadge, NbBlockTitle, NbButton, NbCard, NbCodeBlock, NbContentSlot, NbEyebrow, NbField,
        NbFloatingPanelSlot, NbGrid, NbHeaderBar, NbHero, NbIconButton, NbLinkButton,
        NbModelButton, NbNavLink, NbPage, NbPluginGroup, NbProjectLayout, NbRightSlot, NbShell,
        NbSidebar, NbSidebarToggle, NbSplit, NbTitlebarControls, NbTitlebarNav, NbWorkspace,
        NbWorkspaceBody,
    };
}
