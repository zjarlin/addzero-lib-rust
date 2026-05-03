/// Event system and handler registry (skeleton — to be implemented in #79).
#[derive(Clone)]
pub struct HandlerRegistry;

impl HandlerRegistry {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
