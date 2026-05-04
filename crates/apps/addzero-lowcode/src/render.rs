/// Render pipeline — converts a layout tree into output.
///
/// The actual rendering logic will be fleshed out in #81.
use crate::schema::LayoutSchema;

/// Errors that can occur during rendering.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// The referenced layout does not exist.
    #[error("layout not found: {0}")]
    LayoutNotFound(uuid::Uuid),
    /// A component type in the layout is not registered in the render map.
    #[error("unsupported component type: {0}")]
    UnsupportedComponent(String),
    /// Generic pipeline failure.
    #[error("render pipeline error: {0}")]
    Pipeline(String),
}

/// Placeholder render result.
#[derive(Debug, Clone)]
pub struct RenderOutput {
    /// Generated HTML string.
    pub html: String,
}

/// Renders a layout into a preview-ready output.
///
/// Currently returns a minimal HTML stub; full rendering will be implemented
/// in #81.
pub fn render(_layout: &LayoutSchema) -> Result<RenderOutput, RenderError> {
    Ok(RenderOutput {
        html: "<div class=\"lc-preview\">[render placeholder — #81]</div>".into(),
    })
}
