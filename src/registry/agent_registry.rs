use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;

use crate::adapters::agent::{AgentAdapter, SharedAgentAdapter};

#[derive(Default, Clone)]
pub struct AgentRegistry {
    agents: HashMap<String, SharedAgentAdapter>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<A>(&mut self, name: impl Into<String>, adapter: A) -> Result<(), RegistryError>
    where
        A: AgentAdapter + 'static,
    {
        self.register_shared(name, Arc::new(adapter))
    }

    pub fn register_shared(
        &mut self,
        name: impl Into<String>,
        adapter: SharedAgentAdapter,
    ) -> Result<(), RegistryError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(RegistryError::EmptyName);
        }

        self.agents.insert(name, adapter);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<SharedAgentAdapter> {
        self.agents.get(name).cloned()
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RegistryError {
    #[error("registry names must not be empty")]
    EmptyName,
}
