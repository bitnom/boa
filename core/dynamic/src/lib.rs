//! Dynamic runtime interop capabilities for the Boa JavaScript engine.
//! 
//! This crate provides functionality for dynamically loading and executing JavaScript modules
//! and native functions at runtime, with support for WASM environments.

use boa_engine::{Context, JsResult, JsValue, Module, NativeFunction};
use boa_gc::{Finalize, Trace};
use std::sync::Arc;
use thiserror::Error;
use std::collections::HashSet;

mod context;
mod module;
mod registry;
mod resolver;
mod value;

#[cfg(feature = "wasm")]
mod wasm;

pub use context::DynamicContext;
pub use module::{DynamicModule, IntoDynamicModule};
pub use registry::ModuleRegistry;
pub use resolver::ModuleResolver;
pub use value::DynamicValue;

#[cfg(feature = "wasm")]
pub use wasm::*;

/// Errors that can occur during dynamic operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Error occurred during module registration
    #[error("Failed to register module '{name}': {reason}")]
    ModuleRegistration {
        /// Name of the module that failed to register
        name: String,
        /// Reason for registration failure
        reason: String,
    },

    /// Error occurred during module loading
    #[error("Failed to load module '{name}': {reason}")]
    ModuleLoading {
        /// Name of the module that failed to load
        name: String,
        /// Reason for loading failure
        reason: String,
        /// Source code of the module if available
        #[source]
        module_source: Option<String>,
    },

    /// Error occurred during JavaScript execution
    #[error("JavaScript execution error in '{context}': {message}")]
    Execution {
        /// Context where the error occurred
        context: String,
        /// Error message
        message: String,
        /// Original JavaScript error if available
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Error occurred during type conversion
    #[error("Type conversion error: failed to convert {from} to {to} - {reason}")]
    TypeConversion {
        /// Source type
        from: String,
        /// Target type
        to: String,
        /// Reason for conversion failure
        reason: String,
    },

    /// Circular dependency detected during module loading
    #[error("Circular module dependency detected: {}", dependency_chain.join(" -> "))]
    CircularDependency {
        /// Chain of dependencies that form the cycle
        dependency_chain: Vec<String>,
    },

    /// Module initialization failed
    #[error("Module '{name}' initialization failed: {reason}")]
    ModuleInit {
        /// Name of the module that failed to initialize
        name: String,
        /// Reason for initialization failure
        reason: String,
        /// Original error if available
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Module not found
    #[error("Module '{name}' not found. Available modules: {}", available_modules.join(", "))]
    ModuleNotFound {
        /// Name of the module that was not found
        name: String,
        /// List of available modules
        available_modules: Vec<String>,
        /// Resolved path if attempted
        resolved_path: Option<String>,
    },

    /// Module resolution error
    #[error("Failed to resolve module '{specifier}': {reason}")]
    ModuleResolution {
        /// Module specifier that failed to resolve
        specifier: String,
        /// Reason for resolution failure
        reason: String,
    },

    /// Path resolution error
    #[error("Failed to resolve path '{path}': {reason}")]
    PathResolution {
        /// Path that failed to resolve
        path: String,
        /// Reason for resolution failure
        reason: String,
    },

    /// Invalid module name
    #[error("Invalid module name '{name}': must be a valid identifier")]
    InvalidModuleName {
        /// The invalid module name
        name: String,
    },

    /// Concurrent modification error
    #[error("Concurrent modification error: {0}")]
    ConcurrentModification(String),

    /// WASM-specific error
    #[cfg(feature = "wasm")]
    #[error("WASM error: {message}")]
    WasmError {
        /// Error message
        message: String,
        /// Original error if available
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

/// Result type for dynamic operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Internal helper to convert JsResult to our Result type
pub(crate) fn convert_js_error<T>(result: JsResult<T>) -> Result<T> {
    result.map_err(|e| Error::Execution {
        context: "unknown".to_string(),
        message: e.to_string(),
        source: Some(Box::new(e)),
    })
}

/// Internal helper to convert lock errors to our Error type
pub(crate) fn convert_lock_error<T, E: std::error::Error + Send + Sync + 'static>(result: std::result::Result<T, E>) -> Result<T> {
    result.map_err(|e| Error::ConcurrentModification(e.to_string()))
}

/// Validate if a string is a valid module name
pub(crate) fn is_valid_module_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    
    let first_char = name.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }

    name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}
