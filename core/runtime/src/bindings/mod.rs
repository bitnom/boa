use boa_engine::{Context, JsResult, JsValue, Module, NativeFunction, Object, property::Attribute};
use boa_gc::Trace;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Module resolution hook
pub type ModuleResolver = Arc<dyn Fn(&str) -> Option<String> + Send + Sync>;

/// Module transformation hook
pub type ModuleTransformer = Arc<dyn Fn(String) -> String + Send + Sync>;

/// Runtime bindings configuration for extending JavaScript functionality.
#[derive(Debug, Trace)]
pub struct RuntimeBindings {
    #[unsafe_ignore_trace]
    modules: HashMap<String, ModuleDefinition>,
    #[unsafe_ignore_trace]
    mounted_paths: HashMap<String, PathBuf>,
    #[unsafe_ignore_trace]
    module_resolver: Option<ModuleResolver>,
    #[unsafe_ignore_trace]
    module_transformer: Option<ModuleTransformer>,
    #[unsafe_ignore_trace]
    global_objects: HashMap<String, Object>,
}

/// Definition of a module that can be loaded at runtime.
#[derive(Debug)]
pub struct ModuleDefinition {
    /// Module name
    pub name: String,
    /// Module content as string
    pub content: String,
    /// Native functions to be bound
    pub native_functions: Vec<(String, NativeFunction)>,
}

impl RuntimeBindings {
    /// Create a new RuntimeBindings instance
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            mounted_paths: HashMap::new(),
            module_resolver: None,
            module_transformer: None,
            global_objects: HashMap::new(),
        }
    }

    /// Mount a filesystem path to a virtual path
    pub fn mount_fs(&mut self, virtual_path: impl Into<String>, real_path: impl AsRef<Path>) {
        self.mounted_paths.insert(virtual_path.into(), real_path.as_ref().to_path_buf());
    }

    /// Set module resolver
    pub fn set_module_resolver(&mut self, resolver: impl Fn(&str) -> Option<String> + Send + Sync + 'static) {
        self.module_resolver = Some(Arc::new(resolver));
    }

    /// Set module transformer
    pub fn set_module_transformer(&mut self, transformer: impl Fn(String) -> String + Send + Sync + 'static) {
        self.module_transformer = Some(Arc::new(transformer));
    }

    /// Add a global object
    pub fn add_global_object(&mut self, name: impl Into<String>, obj: Object) {
        self.global_objects.insert(name.into(), obj);
    }

    /// Initialize global objects in context
    pub fn init_globals(&self, context: &mut Context) -> JsResult<()> {
        let global = context.global_object();
        
        for (name, obj) in &self.global_objects {
            global.set(name, obj.clone(), true, context)?;
        }
        
        Ok(())
    }

    /// Register a new module
    pub fn register_module(&mut self, definition: ModuleDefinition) {
        self.modules.insert(definition.name.clone(), definition);
    }

    /// Get a module by name
    pub fn get_module(&self, name: &str) -> Option<&ModuleDefinition> {
        self.modules.get(name)
    }

    /// Resolve module path
    fn resolve_module_path(&self, specifier: &str) -> Option<PathBuf> {
        // Try custom resolver first
        if let Some(resolver) = &self.module_resolver {
            if let Some(resolved) = resolver(specifier) {
                return Some(PathBuf::from(resolved));
            }
        }

        // Try mounted paths
        for (virtual_path, real_path) in &self.mounted_paths {
            if specifier.starts_with(virtual_path) {
                let relative = specifier.strip_prefix(virtual_path).unwrap();
                return Some(real_path.join(relative));
            }
        }

        None
    }

    /// Transform module content
    fn transform_module(&self, content: String) -> String {
        if let Some(transformer) = &self.module_transformer {
            transformer(content)
        } else {
            content
        }
    }

    /// Load and evaluate a module
    pub fn load_module(&self, name: &str, context: &mut Context) -> JsResult<JsValue> {
        // Try registered modules first
        if let Some(module_def) = self.get_module(name) {
            let content = self.transform_module(module_def.content.clone());
            
            // Create module with native functions
            let module = Module::parse(
                content.as_str(),
                Some(name),
                context
            )?;

            // Register native functions
            for (name, func) in &module_def.native_functions {
                module.set_export(name, func.clone().into())?;
            }

            return Ok(module.into());
        }

        // Try resolving from filesystem
        if let Some(path) = self.resolve_module_path(name) {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let content = self.transform_module(content);
                let module = Module::parse(
                    content.as_str(),
                    Some(name),
                    context
                )?;
                return Ok(module.into());
            }
        }

        Ok(JsValue::undefined())
    }
}

impl ModuleDefinition {
    /// Create a new ModuleDefinition
    pub fn new(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            content: content.into(),
            native_functions: Vec::new(),
        }
    }

    /// Add a native function to the module
    pub fn add_native_function(&mut self, name: impl Into<String>, func: NativeFunction) {
        self.native_functions.push((name.into(), func));
    }
}
