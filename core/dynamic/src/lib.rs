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

pub use context::DynamicContext;
pub use module::{DynamicModule, IntoDynamicModule};
pub use registry::ModuleRegistry;
pub use resolver::ModuleResolver;
pub use value::DynamicValue;

/// Errors that can occur during dynamic operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Error occurred during module registration
    #[error("Failed to register module '{name}': {reason}")]
    ModuleRegistration {
        name: String,
        reason: String,
    },

    /// Error occurred during module loading
    #[error("Failed to load module '{name}': {reason}")]
    ModuleLoading {
        name: String,
        reason: String,
    },

    /// Error occurred during JavaScript execution
    #[error("JavaScript execution error in '{context}': {message}")]
    Execution {
        context: String,
        message: String,
    },

    /// Error occurred during type conversion
    #[error("Type conversion error: failed to convert {from} to {to} - {reason}")]
    TypeConversion {
        from: String,
        to: String,
        reason: String,
    },

    /// Circular dependency detected during module loading
    #[error("Circular module dependency detected: {}", dependency_chain.join(" -> "))]
    CircularDependency {
        dependency_chain: Vec<String>,
    },

    /// Module initialization failed
    #[error("Module '{name}' initialization failed: {reason}")]
    ModuleInit {
        name: String,
        reason: String,
    },

    /// Module not found
    #[error("Module '{name}' not found. Available modules: {}", available_modules.join(", "))]
    ModuleNotFound {
        name: String,
        available_modules: Vec<String>,
    },

    /// Module resolution error
    #[error("Failed to resolve module '{specifier}': {reason}")]
    ModuleResolution {
        specifier: String,
        reason: String,
    },

    /// Concurrent modification error
    #[error("Concurrent modification error: {0}")]
    ConcurrentModification(String),
}

/// Result type for dynamic operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Internal helper to convert JsResult to our Result type
pub(crate) fn convert_js_error<T>(result: JsResult<T>) -> Result<T> {
    result.map_err(|e| Error::Execution {
        context: "unknown".to_string(),
        message: e.to_string(),
    })
}

/// Internal helper to convert lock errors to our Error type
pub(crate) fn convert_lock_error<T, E: std::error::Error>(result: std::result::Result<T, E>) -> Result<T> {
    result.map_err(|e| Error::ConcurrentModification(e.to_string()))
}
