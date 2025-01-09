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

    /// Initialize the module in a context
    pub fn initialize(&self, context: &mut Context) -> Result<Module> {
        // Parse module source
        let source = Source::from_bytes(&self.source);
        let module = convert_js_error(Module::parse(source, context))?;

        // Register native functions
        for (name, func) in self.native_functions.iter() {
            convert_js_error(module.set_native_function(name, func.clone(), context))?;
        }

        // Initialize module
        convert_js_error(module.initialize_module(context))
            .map_err(|e| Error::ModuleInit(format!("Failed to initialize module '{}': {}", self.name, e)))?;

        Ok(module)
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
