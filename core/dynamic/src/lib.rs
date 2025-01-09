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
mod value;

pub use context::DynamicContext;
pub use module::{DynamicModule, IntoDynamicModule};
pub use registry::ModuleRegistry;
pub use value::DynamicValue;

/// Errors that can occur during dynamic operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Error occurred during module registration
    #[error("Failed to register module: {0}")]
    ModuleRegistration(String),

    /// Error occurred during module loading
    #[error("Failed to load module: {0}")]
    ModuleLoading(String),

    /// Error occurred during JavaScript execution
    #[error("JavaScript execution error: {0}")]
    Execution(String),

    /// Error occurred during type conversion
    #[error("Type conversion error: {0}")]
    TypeConversion(String),

    /// Circular dependency detected during module loading
    #[error("Circular module dependency detected: {0}")]
    CircularDependency(String),

    /// Module initialization failed
    #[error("Module initialization failed: {0}")]
    ModuleInit(String),

    /// Module not found
    #[error("Module not found: {0}")]
    ModuleNotFound(String),

    /// Concurrent modification error
    #[error("Concurrent modification error: {0}")]
    ConcurrentModification(String),
}

/// Result type for dynamic operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Internal helper to convert JsResult to our Result type
pub(crate) fn convert_js_error<T>(result: JsResult<T>) -> Result<T> {
    result.map_err(|e| Error::Execution(e.to_string()))
}

/// Internal helper to convert lock errors to our Error type
pub(crate) fn convert_lock_error<T, E: std::error::Error>(result: std::result::Result<T, E>) -> Result<T> {
    result.map_err(|e| Error::ConcurrentModification(e.to_string()))
}
