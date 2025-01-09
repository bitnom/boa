use crate::{DynamicModule, Error, Result, convert_lock_error};
use crate::resolver::ModuleResolver;
use boa_engine::{Context, Module};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};

/// A registry for dynamic modules.
#[derive(Debug, Clone, Default)]
pub struct ModuleRegistry {
    /// Map of module specifiers to modules
    modules: Arc<RwLock<HashMap<String, DynamicModule>>>,
    /// Map of modules currently being loaded to their parent modules (for dependency chain tracking)
    loading: Arc<RwLock<HashMap<String, Option<String>>>>,
    /// Module resolver for handling relative imports
    resolver: ModuleResolver,
}

impl ModuleRegistry {
    /// Create a new module registry
    pub fn new() -> Self {
        Self {
            modules: Arc::new(RwLock::new(HashMap::new())),
            loading: Arc::new(RwLock::new(HashMap::new())),
            resolver: ModuleResolver::new(),
        }
    }

    /// Add a base path for module resolution
    pub fn add_base_path<P: AsRef<std::path::Path>>(&mut self, path: P) {
        self.resolver.add_base_path(path);
    }

    /// Register a module
    pub fn register(&self, module: DynamicModule) -> Result<()> {
        let mut modules = convert_lock_error(self.modules.write())?;
        modules.insert(module.name().to_string(), module);
        Ok(())
    }

    /// Load a module by its specifier
    pub async fn load_module(&self, specifier: &str, context: &mut Context) -> Result<Module> {
        // Get the current module from the loading chain (if any)
        let parent_module = if let Ok(loading) = self.loading.read() {
            loading.iter()
                .find(|(_, parent)| parent.is_none())
                .map(|(name, _)| name.clone())
        } else {
            None
        };

        // Resolve the module specifier
        let resolved_specifier = self.resolver.resolve(specifier, parent_module.as_deref())?;

        // Get the module while holding read lock
        let module = {
            let modules = convert_lock_error(self.modules.read())?;
            modules.get(&resolved_specifier)
                .or_else(|| modules.get(specifier))
                .ok_or_else(|| Error::ModuleNotFound {
                    name: specifier.to_string(),
                    available_modules: modules.keys().cloned().collect(),
                })?
                .clone()
        };

        // Initialize the module (no locks held)
        module.initialize_async(context).await
            .map_err(|e| Error::ModuleLoading {
                name: specifier.to_string(),
                reason: e.to_string(),
            })
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
            .map(|loading| loading.contains_key(specifier))
            .unwrap_or(false)
    }

    /// Get the full dependency chain for a module being loaded
    pub fn get_loading_chain(&self, specifier: &str) -> Result<Vec<String>> {
        let loading = convert_lock_error(self.loading.read())?;
        let mut chain = Vec::new();
        let mut current = Some(specifier.to_string());

        while let Some(module) = current {
            chain.push(module.clone());
            current = loading.get(&module).and_then(|parent| parent.clone());
        }

        chain.reverse();
        Ok(chain)
    }

    /// Mark a module as being loaded, with optional parent module
    pub fn mark_loading(&self, specifier: &str, parent: Option<&str>) -> Result<()> {
        let mut loading = convert_lock_error(self.loading.write())?;
        loading.insert(
            specifier.to_string(),
            parent.map(|p| p.to_string())
        );
        Ok(())
    }

    /// Unmark a module as being loaded
    pub fn unmark_loading(&self, specifier: &str) -> Result<()> {
        let mut loading = convert_lock_error(self.loading.write())?;
        loading.remove(specifier);
        Ok(())
    }

    /// Clear all modules and loading states
    pub fn clear(&self) -> Result<()> {
        {
            let mut modules = convert_lock_error(self.modules.write())?;
            modules.clear();
        }
        {
            let mut loading = convert_lock_error(self.loading.write())?;
            loading.clear();
        }
        Ok(())
    }

    /// Get all registered module names
    pub fn module_names(&self) -> Result<Vec<String>> {
        let modules = convert_lock_error(self.modules.read())?;
        Ok(modules.keys().cloned().collect())
    }

    /// Check for circular dependencies in the current loading chain
    pub fn check_circular_dependencies(&self, specifier: &str) -> Result<()> {
        let loading = convert_lock_error(self.loading.read())?;
        let mut visited = HashSet::new();
        let mut current = Some(specifier.to_string());

        while let Some(module) = current {
            if !visited.insert(module.clone()) {
                return Err(Error::CircularDependency {
                    dependency_chain: self.get_loading_chain(&module)?,
                });
            }
            current = loading.get(&module).and_then(|parent| parent.clone());
        }

        Ok(())
    }
}
