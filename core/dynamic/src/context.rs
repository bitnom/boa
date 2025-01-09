use crate::{Error, ModuleRegistry, Result, convert_js_error, convert_lock_error};
use boa_engine::{Context, JsValue, Module, Source, NativeFunction};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use futures::Future;

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

        let result = self.registry.load_module(specifier, &mut self.inner).await;

        // Unmark module as loading
        self.registry.unmark_loading(specifier)?;

        // Cache successful result
        if let Ok(ref module) = result {
            if let Ok(mut cache) = self.module_cache.write() {
                cache.insert(specifier.to_string(), module.clone());
            }
        }

        result
    }

    /// Clear the module cache
    pub fn clear_cache(&mut self) -> Result<()> {
        if let Ok(mut cache) = self.module_cache.write() {
            cache.clear();
            Ok(())
        } else {
            Err(Error::ConcurrentModification("Failed to acquire cache write lock".to_string()))
        }
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
