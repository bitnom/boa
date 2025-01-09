use crate::{Error, ModuleRegistry, Result, convert_js_error, convert_lock_error};
use boa_engine::{Context, JsValue, Module, Source, NativeFunction};
use boa_gc::{Finalize, Trace};
use std::sync::Arc;

/// An extended context that supports dynamic module loading and native function registration.
#[derive(Debug, Clone)]
pub struct DynamicContext {
    /// The underlying Boa context
    inner: Context,
    /// Registry for dynamic modules
    registry: ModuleRegistry,
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
        }
    }

    /// Register a module at runtime
    pub fn register_module<T>(&mut self, name: &str, module: T) -> Result<()>
    where
        T: crate::IntoDynamicModule,
    {
        // Check if module is already being loaded (circular dependency)
        if self.registry.is_loading(name) {
            return Err(Error::CircularDependency(name.to_string()));
        }

        module.register(self).map_err(|e| Error::ModuleRegistration(e.to_string()))
    }

    /// Load a module at runtime
    pub async fn load_module(&mut self, specifier: &str) -> Result<Module> {
        // Check for circular dependencies
        if self.registry.is_loading(specifier) {
            return Err(Error::CircularDependency(specifier.to_string()));
        }

        // Mark module as loading
        self.registry.mark_loading(specifier)?;

        // Use drop guard to ensure we always unmark loading state
        struct LoadGuard<'a> {
            registry: &'a ModuleRegistry,
            specifier: String,
        }

        impl<'a> Drop for LoadGuard<'a> {
            fn drop(&mut self) {
                let _ = self.registry.unmark_loading(&self.specifier);
            }
        }

        let _guard = LoadGuard {
            registry: &self.registry,
            specifier: specifier.to_string(),
        };

        // Load the module
        self.registry.load_module(specifier, &mut self.inner).await
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
