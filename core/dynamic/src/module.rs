use crate::{DynamicContext, Error, Result, convert_js_error};
use boa_engine::{Context, JsValue, Module, NativeFunction, Source};
use boa_gc::{Finalize, Trace, GcRef};
use std::sync::Arc;

/// A dynamic module that can be loaded at runtime.
#[derive(Debug, Clone)]
pub struct DynamicModule {
    /// The module's name/specifier
    name: String,
    /// The module's source code
    source: String,
    /// Native functions exposed by this module
    native_functions: Arc<Vec<(String, NativeFunction)>>,
}

unsafe impl Finalize for DynamicModule {}

unsafe impl Trace for DynamicModule {
    fn trace(&self, visitor: &mut boa_gc::Visitor) {
        // Trace native functions
        for (_, func) in self.native_functions.iter() {
            func.trace(visitor);
        }
    }
}

impl DynamicModule {
    /// Create a new dynamic module
    pub fn new(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            source: source.into(),
            native_functions: Arc::new(Vec::new()),
        }
    }

    /// Add a native function to the module
    pub fn add_native_function(&mut self, name: impl Into<String>, func: NativeFunction) {
        // Clone the Arc and create a new Vec with the additional function
        let mut functions = (*self.native_functions).clone();
        functions.push((name.into(), func));
        self.native_functions = Arc::new(functions);
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

    /// Initialize the module with cleanup on failure
    pub fn initialize(&self, context: &mut Context) -> Result<Module> {
        let mut guard = InitGuard::new(self.name.clone(), context);
        
        let result = self.do_initialize(context);
        if result.is_ok() {
            guard.commit();
        }
        
        result
    }

    /// Async initialization support
    pub async fn initialize_async(&self, context: &mut Context) -> Result<Module> {
        let module = self.initialize(context)?;
        
        // Check for async initialization function
        if let Ok(init_fn) = module.get_property("asyncInit", context) {
            if init_fn.is_callable() {
                let result = init_fn.call(&[], context)?;
                if let Some(promise) = result.as_promise() {
                    // Wait for async initialization to complete
                    promise.await?;
                }
            }
        }
        
        Ok(module)
    }

    /// Internal initialization implementation
    fn do_initialize(&self, context: &mut Context) -> Result<Module> {
        // Parse module source
        let module = Module::parse(Source::from_bytes(&self.source), context)
            .map_err(|e| Error::ModuleInit {
                name: self.name.clone(),
                reason: e.to_string(),
            })?;

        // Register native functions
        for (name, func) in self.native_functions.iter() {
            module.add_native_function(name, func.clone(), context)
                .map_err(|e| Error::ModuleInit {
                    name: self.name.clone(),
                    reason: format!("Failed to register native function '{}': {}", name, e),
                })?;
        }

        Ok(module)
    }
}

/// A guard to ensure proper cleanup if initialization fails
struct InitGuard<'a> {
    name: String,
    context: &'a mut Context,
    committed: bool,
}

impl<'a> InitGuard<'a> {
    fn new(name: String, context: &'a mut Context) -> Self {
        Self {
            name,
            context,
            committed: false,
        }
    }

    fn commit(&mut self) {
        self.committed = true;
    }
}

impl<'a> Drop for InitGuard<'a> {
    fn drop(&mut self) {
        if !self.committed {
            // Cleanup if initialization failed
            self.context.gc();
        }
    }
}

/// A trait for types that can be converted into a dynamic module.
pub trait IntoDynamicModule {
    /// Register this module with a dynamic context.
    fn register(&self, context: &mut DynamicContext) -> Result<()>;
}

impl IntoDynamicModule for DynamicModule {
    fn register(&self, context: &mut DynamicContext) -> Result<()> {
        // Initialize the module
        let module = self.initialize(context.inner_mut())?;

        // Register it with the registry
        context.registry.register(self.clone())
    }
}
