//! Dynamic context for JavaScript execution with module support.
//!
//! This module provides the [`DynamicContext`] type, which extends Boa's standard
//! context with support for dynamic module loading and native function registration.
//! It handles module caching, dependency tracking, and provides a safe interface
//! for executing JavaScript code with access to loaded modules.
//!
//! # Examples
//!
//! ```rust
//! use boa_dynamic::DynamicContext;
//!
//! # async fn example() -> boa_dynamic::Result<()> {
//! let mut context = DynamicContext::new();
//!
//! // Register a module
//! context.register_module("math", r#"
//!     export function add(a, b) {
//!         return a + b;
//!     }
//! "#)?;
//!
//! // Load and use the module
//! let result = context.evaluate(r#"
//!     import { add } from 'math';
//!     add(2, 3);
//! "#)?;
//!
//! assert_eq!(result.to_string(), "5");
//! # Ok(())
//! # }
//! ```

use crate::{Error, ModuleRegistry, Result, convert_js_error, convert_lock_error, is_valid_module_name};
use boa_engine::{Context, JsValue, Module, Source, NativeFunction};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use futures::Future;
use std::path::Path;

/// An extended context that supports dynamic module loading and native function registration.
///
/// The `DynamicContext` wraps Boa's standard [`Context`] and adds support for:
/// - Dynamic module loading and registration
/// - Module caching and dependency tracking
/// - Safe concurrent access to shared resources
/// - Platform-independent path resolution
///
/// # Thread Safety
///
/// The context uses interior mutability with `RwLock` to ensure thread-safe access
/// to shared resources like the module cache and registry. This allows multiple
/// contexts to safely share modules while preventing data races.
///
/// # Module Loading
///
/// Modules can be loaded in several ways:
/// - From strings using `register_module`
/// - From files using relative or absolute paths
/// - From predefined module names
///
/// The context tracks module dependencies and prevents circular dependencies.
///
/// # Caching
///
/// Loaded modules are cached to improve performance. The cache can be cleared
/// using the `clear_cache` method if needed.
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
    /// Create a new dynamic context with default settings.
    ///
    /// This creates a fresh context with:
    /// - An empty module registry
    /// - An empty module cache
    /// - Default Boa context settings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use boa_dynamic::DynamicContext;
    ///
    /// let context = DynamicContext::new();
    /// ```
    pub fn new() -> Self {
        Self {
            inner: Context::default(),
            registry: ModuleRegistry::new(),
            module_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a module at runtime
    ///
    /// This method registers a module with the given `name` and `module` code.
    /// The module is validated and checked for circular dependencies before
    /// registration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The module name is invalid
    /// - The module is already being loaded
    /// - The module registration fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use boa_dynamic::DynamicContext;
    ///
    /// let mut context = DynamicContext::new();
    /// context.register_module("math", r#"
    ///     export function add(a, b) {
    ///         return a + b;
    ///     }
    /// "#)?;
    /// ```
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
    ///
    /// This method loads a module with the given `specifier` and returns the
    /// loaded module. The module is cached to improve performance and prevent
    /// duplicate loads.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The module is already being loaded
    /// - The module loading fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use boa_dynamic::DynamicContext;
    ///
    /// let mut context = DynamicContext::new();
    /// let module = context.load_module("math").await?;
    /// ```
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
    ///
    /// This method imports a module with the given `specifier` and returns the
    /// imported module. The module is loaded and cached if necessary.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The module is already being loaded
    /// - The module loading fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use boa_dynamic::DynamicContext;
    ///
    /// let mut context = DynamicContext::new();
    /// let module = context.import("math").await?;
    /// ```
    pub async fn import(&mut self, specifier: &str) -> Result<Module> {
        self.load_module(specifier).await
    }

    /// Clear the module cache and invalidate loaded modules
    ///
    /// This method clears the module cache and invalidates all loaded modules.
    /// This can be useful for testing or when the module cache needs to be
    /// refreshed.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache clearing fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use boa_dynamic::DynamicContext;
    ///
    /// let mut context = DynamicContext::new();
    /// context.clear_cache()?;
    /// ```
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
    ///
    /// This method evaluates the given `code` with access to the loaded modules.
    /// The code is executed in the context of the current module.
    ///
    /// # Errors
    ///
    /// Returns an error if the code evaluation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use boa_dynamic::DynamicContext;
    ///
    /// let mut context = DynamicContext::new();
    /// let result = context.evaluate("console.log('Hello World!');")?;
    /// ```
    pub fn evaluate(&mut self, code: &str) -> Result<JsValue> {
        let source = Source::from_bytes(code);
        convert_js_error(self.inner.eval(source))
    }

    /// Get a reference to the inner context
    ///
    /// This method returns a reference to the inner Boa context.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use boa_dynamic::DynamicContext;
    ///
    /// let context = DynamicContext::new();
    /// let inner = context.inner();
    /// ```
    pub fn inner(&self) -> &Context {
        &self.inner
    }

    /// Get a mutable reference to the inner context
    ///
    /// This method returns a mutable reference to the inner Boa context.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use boa_dynamic::DynamicContext;
    ///
    /// let mut context = DynamicContext::new();
    /// let inner = context.inner_mut();
    /// ```
    pub fn inner_mut(&mut self) -> &mut Context {
        &mut self.inner
    }

    /// Add a base path for module resolution
    ///
    /// This method adds a base path for module resolution. The base path is used
    /// to resolve relative module specifiers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use boa_dynamic::DynamicContext;
    ///
    /// let mut context = DynamicContext::new();
    /// context.add_base_path("/path/to/modules");
    /// ```
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
