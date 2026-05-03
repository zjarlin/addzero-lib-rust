pub mod schema;
pub mod grid;
pub mod registry;
pub mod editor;
pub mod events;
pub mod scripting;
pub mod render;
pub mod template;
pub mod repo;
pub mod router;
pub mod state;

// Re-export core schema types
pub use schema::{
    ComponentDefRecord, ComponentNode, EventBindingRecord, GridArea, GridDefinition,
    HandlerType, LayoutSchema,
};

// Re-export repository trait and record
pub use repo::{LayoutRecord, LayoutRepository, PgLayoutRepo, RepoError};

pub use router::lowcode_router;
pub use state::LowcodeState;
