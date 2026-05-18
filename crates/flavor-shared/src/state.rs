#[derive(Debug, Clone)]
pub struct PluginState<T> {
    config: T,
}

impl<T> PluginState<T> {
    pub fn new(config: T) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &T {
        &self.config
    }
}
