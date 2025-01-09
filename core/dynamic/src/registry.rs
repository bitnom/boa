use crate::{DynamicModule, Error, Result};
use boa_engine::{Context, Module};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A registry for dynamic modules.
#[derive(Debug, Default)]
pub struct ModuleRegistry {
    /// Map of module specifiers to modules
    modules: Arc<RwLock<HashMap<String, DynamicModule>>>,
}

impl ModuleRegistry {
    /// Create a new module registry
    pub fn new() -> Self {
        Self {
            modules: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a module
    pub fn register(&self, module: DynamicModule) -> Result<()> {
        let mut modules = self.modules.write().map_err(|e| Error::ModuleRegistration(e.to_string()))?;
        modules.insert(module.name().to_string(), module);
        Ok(())
    }

    /// Load a module by its specifier
    pub async fn load_module(&self, specifier: &str, context: &mut Context) -> Result<Module> {
        let modules = self.modules.read().map_err(|e| Error::ModuleLoading(e.to_string()))?;
        
        let module = modules
            .get(specifier)
            .ok_or_else(|| Error::ModuleLoading(format!("Module not found: {}", specifier)))?;

        // Create a new module from the source
        let module = Module::parse(module.source(), Some(specifier), context)
            .map_err(|e| Error::ModuleLoading(e.to_string()))?;

        Ok(module)
    }

    /// Check if a module is registered
    pub fn has_module(&self, specifier: &str) -> bool {
        self.modules
            .read()
            .map(|modules| modules.contains_key(specifier))
            .unwrap_or(false)
    }
}
