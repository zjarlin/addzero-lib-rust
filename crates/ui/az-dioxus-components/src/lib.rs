#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

mod class_name;

/// Card primitives with the `az-card` class contract.
pub mod az_card;
/// Table primitives with the `az-table` class contract.
pub mod az_table;

/// Convenience exports for composing the public UI primitives.
pub mod prelude {
    //! Convenience exports for the public Dioxus components in this crate.

    pub use crate::az_card::AzCard;
    pub use crate::az_table::{
        AzTable, AzTableBody, AzTableCaption, AzTableCell, AzTableFooter, AzTableHead,
        AzTableHeaderCell, AzTableRow,
    };
}
