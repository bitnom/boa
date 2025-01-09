use crate::{Error, ModuleRegistry, Result, convert_js_error, convert_lock_error, is_valid_module_name};
use boa_engine::{Context, JsValue, Module, Source, NativeFunction};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use futures::Future;
use std::path::Path;

/// An extended context that supports dynamic module loading and native function registration.
#[derive(Debug, Clone)]
pub struct DynamicContext {
    /// The underlying Boa context
    inner: Context,
    /// Registry for dynamic modules
    registry: ModuleRegistry,
    /// Cache for loaded modules
    module_cache: Arc<RwLock<HashMap<String, Module>>>,
}

unsafe impl Finalize for DynamicContext {}

unsafe impl Trace for DynamicContext {
    fn trace(&self, visitor: &mut boa_gc::Visitor) {
        self.inner.trace(visitor);
    }
}

impl DynamicContext {
    /// Create a new dynamic context
    pub fn new() -> Self {
        Self {
            inner: Context::default(),
            registry: ModuleRegistry::new(),
            module_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a module at runtime
    pub fn register_module<T>(&mut self, name: &str, module: T) -> Result<()>
    where
        T: crate::IntoDynamicModule,
    {
        // Validate module name
        if !is_valid_module_name(name) {
            return Err(Error::InvalidModuleName {
                name: name.to_string(),
            });
        }

        // Check if module is already being loaded
        if self.registry.is_loading(name) {
            return Err(Error::CircularDependency {
                dependency_chain: vec![name.to_string()],
            });
        }

        // Clear module from cache if it exists
        if let Ok(mut cache) = self.module_cache.write() {
            cache.remove(name);
        }

        module.register(self).map_err(|e| Error::ModuleRegistration {
            name: name.to_string(),
            reason: e.to_string(),
        })
    }

    /// Load a module at runtime with caching and dependency tracking
    pub async fn load_module(&mut self, specifier: &str) -> Result<Module> {
        // Check cache first
        if let Ok(cache) = self.module_cache.read() {
            if let Some(module) = cache.get(specifier) {
                return Ok(module.clone());
            }
        }

        // Check for circular dependencies
        self.registry.check_circular_dependencies(specifier)?;

        // Get the current module from the loading chain (if any)
        let current_module = if let Ok(loading) = self.registry.get_loading_chain(specifier) {
            loading.last().cloned()
        } else {
            None
        };

        // Mark module as loading with parent information
        self.registry.mark_loading(specifier, current_module.as_deref())?;

        // Create a guard that will unmark the module as loading when dropped
        struct LoadingGuard<'a> {
            registry: &'a ModuleRegistry,
            specifier: String,
        }

        impl<'a> Drop for LoadingGuard<'a> {
            fn drop(&mut self) {
                let _ = self.registry.unmark_loading(&self.specifier);
            }
        }

        let _guard = LoadingGuard {
            registry: &self.registry,
            specifier: specifier.to_string(),
        };

        // Load the module
        let result = self.registry.load_module(specifier, &mut self.inner).await;

        match &result {
            Ok(module) => {
                // Cache successful result
                if let Ok(mut cache) = self.module_cache.write() {
                    cache.insert(specifier.to_string(), module.clone());
                }
            }
            Err(e) => {
                // Log error details for debugging
                eprintln!("Failed to load module '{}': {}", specifier, e);
            }
        }

        result
    }

    /// Import a module using a relative or absolute specifier
    pub async fn import(&mut self, specifier: &str) -> Result<Module> {
        self.load_module(specifier).await
    }

    /// Clear the module cache and invalidate loaded modules
    pub fn clear_cache(&mut self) -> Result<()> {
        // Clear the module cache
        if let Ok(mut cache) = self.module_cache.write() {
            cache.clear();
        }

        // Clear the registry
        self.registry.clear()?;

        Ok(())
    }

    /// Evaluate JavaScript code with access to dynamic modules
    pub fn evaluate(&mut self, code: &str) -> Result<JsValue> {
        let source = Source::from_bytes(code);
        convert_js_error(self.inner.eval(source))
    }

    /// Get a reference to the inner context
    pub fn inner(&self) -> &Context {
        &self.inner
    }

    /// Get a mutable reference to the inner context
    pub fn inner_mut(&mut self) -> &mut Context {
        &mut self.inner
    }

    /// Add a base path for module resolution
    pub fn add_base_path<P: AsRef<Path>>(&mut self, path: P) {
        self.registry.add_base_path(path);
    }
}

impl Default for DynamicContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DynamicContext {
    fn drop(&mut self) {
        // Clean up any resources
        self.registry.clear();
    }
}
