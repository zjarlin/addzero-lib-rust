pub mod editor;
pub mod events;
pub mod grid;
pub mod registry;
pub mod render;
pub mod repo;
pub mod router;
pub mod schema;
pub mod scripting;
pub mod state;
pub mod template;

// Re-export core schema types
pub use grid::{DEFAULT_COLUMNS, GridEngine, compile_css};
pub use schema::{
    Breakpoint, ComponentDefRecord, ComponentNode, EventBindingRecord, GridArea, GridDefinition,
    HandlerType, LayoutSchema,
};

// Re-export repository trait and record
pub use repo::{LayoutRecord, LayoutRepository, PgLayoutRepo, RepoError};

pub use router::lowcode_router;
pub use state::LowcodeState;
