/// Render pipeline — converts a layout tree into output (skeleton — to be implemented in #81).

use crate::schema::LayoutSchema;

/// Errors that can occur during rendering.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("layout not found: {0}")]
    LayoutNotFound(uuid::Uuid),
    #[error("unsupported component type: {0}")]
    UnsupportedComponent(String),
    #[error("render pipeline error: {0}")]
    Pipeline(String),
}

/// Placeholder render result.
#[derive(Debug, Clone)]
pub struct RenderOutput {
    pub html: String,
}

/// Renders a layout into a preview-ready output.
///
/// The actual rendering logic will be fleshed out in #81.
pub fn render(_layout: &LayoutSchema) -> Result<RenderOutput, RenderError> {
    todo!("render pipeline — will be implemented in #81")
}
