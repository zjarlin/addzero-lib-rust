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
pub use schema::{
    ComponentDefRecord, ComponentNode, EventBindingRecord, GridArea, GridDefinition, HandlerType,
    LayoutSchema,
};

// Re-export repository trait and record
pub use repo::{LayoutRecord, LayoutRepository, PgLayoutRepo, RepoError};

// Re-export registry types
pub use registry::{ComponentEntry, ComponentInfo, ComponentRegistry, RegistryError};

// Re-export editor types
pub use editor::{EditorError, LayoutEditor};

pub use router::lowcode_router;
pub use state::LowcodeState;
