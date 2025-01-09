use crate::{Error, ModuleRegistry, Result};
use boa_engine::{Context, JsResult, JsValue, Module, NativeFunction};
use std::sync::Arc;

/// An extended context that supports dynamic module loading and native function registration.
#[derive(Debug)]
pub struct DynamicContext {
    /// The underlying Boa context
    inner: Context,
    /// Registry for dynamic modules
    registry: ModuleRegistry,
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
        module.register(self).map_err(|e| Error::ModuleRegistration(e.to_string()))
    }

    /// Load a module at runtime
    pub async fn load_module(&mut self, specifier: &str) -> Result<Module> {
        self.registry
            .load_module(specifier, &mut self.inner)
            .await
            .map_err(|e| Error::ModuleLoading(e.to_string()))
    }

    /// Evaluate JavaScript code with access to dynamic modules
    pub fn evaluate(&mut self, code: &str) -> Result<JsValue> {
        self.inner
            .eval(code)
            .map_err(|e| Error::Execution(e.to_string()))
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
