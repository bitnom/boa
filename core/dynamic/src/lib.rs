//! Dynamic runtime interop capabilities for the Boa JavaScript engine.
//! 
//! This crate provides functionality for dynamically loading and executing JavaScript modules
//! and native functions at runtime, with support for WASM environments.

use boa_engine::{Context, JsResult, JsValue, Module, NativeFunction};
use boa_gc::{Finalize, Trace};
use std::sync::Arc;
use thiserror::Error;

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
}

/// Result type for dynamic operations.
pub type Result<T> = std::result::Result<T, Error>;
