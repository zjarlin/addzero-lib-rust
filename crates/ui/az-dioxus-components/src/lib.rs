#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod class_name;
mod component_style;
mod style;

/// Accordion primitives for dense workbench panels.
pub mod accordion;
/// Structured table components built on the table primitives.
pub mod data_table;
/// Compact form primitives for dense workbench pages.
pub mod form;
/// Grammar-style search input and parsing model.
pub mod grammar_search;
/// Neobrutalism-inspired SSR primitives.
pub mod neobrutal;
/// Small status badge primitives.
pub mod status_badge;
/// Lightweight surface card primitive.
pub mod surface_card;
/// Semantic table primitives.
pub mod table;
/// Toolbar button and link primitives.
pub mod toolbar_button;
/// Dense workbench layout primitives for admin-style pages.
pub mod workbench;

/// 组合公开 UI 基础组件时使用的便捷导出。
pub mod prelude {
    pub use crate::accordion::Accordion;
    pub use crate::data_table::{
        DataTable, DataTableAlign, DataTableCell, DataTableColumn, DataTableRow,
    };
    pub use crate::form::{
        ActionForm, CheckboxRow, FormGrid, FormRow, HiddenInput, Input, Select, SelectOption,
    };
    pub use crate::grammar_search::{
        GrammarSearchField, GrammarSearchFilter, GrammarSearchInput, GrammarSearchQuery,
        GrammarSearchTerm, parse_grammar_search_query,
    };
    pub use crate::neobrutal::{
        Badge, BlockTitle, Button, Card, CodeBlock, ContentSlot, Eyebrow, Field, FloatingPanelSlot,
        Grid, HeaderBar, Hero, IconButton, LinkButton, ModelButton, NavLink, Page, PluginGroup,
        ProjectLayout, RightSlot, Shell, Sidebar, SidebarToggle, Split, TitlebarControls,
        TitlebarNav, Workspace, WorkspaceBody,
    };
    pub use crate::status_badge::{StatusBadge, StatusBadgeTone};
    pub use crate::surface_card::SurfaceCard;
    pub use crate::table::{
        Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeaderCell,
        TableRow,
    };
    pub use crate::toolbar_button::{ToolbarButton, ToolbarButtonLink, ToolbarButtonTone};
    pub use crate::workbench::{
        PageHeader, SplitWorkbench, TableViewport, Toolbar, WorkbenchDetail, WorkbenchDetailHeader,
        WorkbenchPage, WorkbenchTree, WorkbenchTreeHeader, WorkbenchTreeList,
    };
}
