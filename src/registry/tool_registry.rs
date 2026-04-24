use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;

use crate::adapters::tool::{SharedToolAdapter, ToolAdapter};

#[derive(Default, Clone)]
pub struct ToolRegistry {
    tools: HashMap<String, SharedToolAdapter>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<T>(&mut self, name: impl Into<String>, adapter: T) -> Result<(), RegistryError>
    where
        T: ToolAdapter + 'static,
    {
        self.register_shared(name, Arc::new(adapter))
    }

    pub fn register_shared(
        &mut self,
        name: impl Into<String>,
        adapter: SharedToolAdapter,
    ) -> Result<(), RegistryError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(RegistryError::EmptyName);
        }

        self.tools.insert(name, adapter);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<SharedToolAdapter> {
        self.tools.get(name).cloned()
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RegistryError {
    #[error("registry names must not be empty")]
    EmptyName,
}
