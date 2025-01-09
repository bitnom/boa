use crate::{DynamicContext, Error, Result};
use boa_engine::{Context, JsResult, JsValue, Module, NativeFunction};
use boa_gc::{Finalize, Trace};
use std::sync::Arc;

/// A dynamic module that can be loaded at runtime.
#[derive(Debug)]
pub struct DynamicModule {
    /// The module's name/specifier
    name: String,
    /// The module's source code
    source: String,
    /// Native functions exposed by this module
    native_functions: Vec<(String, NativeFunction)>,
}

impl DynamicModule {
    /// Create a new dynamic module
    pub fn new(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            source: source.into(),
            native_functions: Vec::new(),
        }
    }

    /// Add a native function to the module
    pub fn add_native_function(&mut self, name: impl Into<String>, func: NativeFunction) {
        self.native_functions.push((name.into(), func));
    }

    /// Get the module's name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the module's source code
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get the module's native functions
    pub fn native_functions(&self) -> &[(String, NativeFunction)] {
        &self.native_functions
    }
}

/// A trait for types that can be converted into a dynamic module.
pub trait IntoDynamicModule {
    /// Register this module with a dynamic context.
    fn register(&self, context: &mut DynamicContext) -> Result<()>;
}

impl IntoDynamicModule for DynamicModule {
    fn register(&self, context: &mut DynamicContext) -> Result<()> {
        // First register any native functions
        for (name, func) in &self.native_functions {
            context
                .inner_mut()
                .register_global_function(name, func.clone())
                .map_err(|e| Error::ModuleRegistration(e.to_string()))?;
        }

        // Then evaluate the module's source code
        context.evaluate(&self.source)?;
        Ok(())
    }
}
