use crate::{DynamicModule, Error, Result, convert_lock_error};
use boa_engine::{Context, Module};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// A registry for dynamic modules.
#[derive(Debug, Clone, Default)]
pub struct ModuleRegistry {
    /// Map of module specifiers to modules
    modules: Arc<RwLock<HashMap<String, DynamicModule>>>,
    /// Set of modules currently being loaded (for circular dependency detection)
    loading: Arc<RwLock<HashSet<String>>>,
}

impl ModuleRegistry {
    /// Create a new module registry
    pub fn new() -> Self {
        Self {
            modules: Arc::new(RwLock::new(HashMap::new())),
            loading: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Register a module
    pub fn register(&self, module: DynamicModule) -> Result<()> {
        let mut modules = convert_lock_error(self.modules.write())?;
        modules.insert(module.name().to_string(), module);
        Ok(())
    }

    /// Load a module by its specifier
    pub async fn load_module(&self, specifier: &str, context: &mut Context) -> Result<Module> {
        // Get the module while holding read lock
        let module = {
            let modules = convert_lock_error(self.modules.read())?;
            modules.get(specifier)
                .ok_or_else(|| Error::ModuleNotFound(specifier.to_string()))?
                .clone()
        };

        // Initialize the module (no locks held)
        module.initialize(context)
    }

    /// Check if a module is registered
    pub fn has_module(&self, specifier: &str) -> bool {
        self.modules
            .read()
            .map(|modules| modules.contains_key(specifier))
            .unwrap_or(false)
    }

    /// Check if a module is currently being loaded
    pub fn is_loading(&self, specifier: &str) -> bool {
        self.loading
            .read()
            .map(|loading| loading.contains(specifier))
            .unwrap_or(false)
    }

    /// Mark a module as being loaded
    pub(crate) fn mark_loading(&self, specifier: &str) -> Result<()> {
        let mut loading = convert_lock_error(self.loading.write())?;
        if !loading.insert(specifier.to_string()) {
            return Err(Error::CircularDependency(format!(
                "Module '{}' is already being loaded", specifier
            )));
        }
        Ok(())
    }

    /// Unmark a module as being loaded
    pub(crate) fn unmark_loading(&self, specifier: &str) -> Result<()> {
        let mut loading = convert_lock_error(self.loading.write())?;
        if !loading.remove(specifier) {
            return Err(Error::ModuleNotFound(format!(
                "Module '{}' was not marked as loading", specifier
            )));
        }
        Ok(())
    }

    /// Clear all modules and loading states
    pub fn clear(&self) {
        if let Ok(mut modules) = self.modules.write() {
            modules.clear();
        }
        if let Ok(mut loading) = self.loading.write() {
            loading.clear();
        }
    }
}
