//! WASM-specific functionality for the dynamic module system.
//!
//! This module provides WASM bindings and utilities for the dynamic module system.
//! It is only available when the `wasm` feature is enabled.

#[cfg(feature = "wasm")]
use {
    crate::{DynamicContext, Error, Result},
    wasm_bindgen::prelude::*,
    js_sys::{Function, Object, Promise},
    web_sys::{Window, Performance},
};

#[cfg(feature = "wasm")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// WASM-specific extension traits for DynamicContext
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl DynamicContext {
    /// Create a new DynamicContext instance in WASM
    #[wasm_bindgen(constructor)]
    pub fn new_wasm() -> Result<DynamicContext> {
        Ok(DynamicContext::new())
    }

    /// Evaluate JavaScript code and return a Promise
    #[wasm_bindgen(js_name = evaluateAsync)]
    pub fn evaluate_async(&mut self, code: &str) -> Result<Promise> {
        let result = self.evaluate(code)?;
        Ok(Promise::resolve(&result.into()))
    }

    /// Register a JavaScript function as a module
    #[wasm_bindgen(js_name = registerFunction)]
    pub fn register_function(&mut self, name: &str, func: &Function) -> Result<()> {
        let module = Object::new();
        js_sys::Reflect::set(&module, &"default".into(), func)
            .map_err(|e| Error::WasmError {
                message: "Failed to set function as default export".to_string(),
                source: Some(Box::new(e)),
            })?;

        self.register_module(name, module)
    }

    /// Get performance metrics
    #[wasm_bindgen(js_name = getPerformance)]
    pub fn get_performance(&self) -> Result<Performance> {
        let window = web_sys::window()
            .ok_or_else(|| Error::WasmError {
                message: "No window object available".to_string(),
                source: None,
            })?;

        Ok(window.performance()
            .ok_or_else(|| Error::WasmError {
                message: "No performance object available".to_string(),
                source: None,
            })?)
    }
}

#[cfg(all(test, feature = "wasm"))]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_wasm_evaluation() {
        let mut context = DynamicContext::new_wasm().unwrap();
        let result = context.evaluate("2 + 2").unwrap();
        assert_eq!(result.as_f64().unwrap(), 4.0);
    }

    #[wasm_bindgen_test]
    fn test_function_registration() {
        let mut context = DynamicContext::new_wasm().unwrap();
        let func = Function::new_with_args("a, b", "return a + b");
        context.register_function("add", &func).unwrap();
        
        let result = context.evaluate("
            import { default as add } from 'add';
            add(2, 3)
        ").unwrap();
        
        assert_eq!(result.as_f64().unwrap(), 5.0);
    }

    #[wasm_bindgen_test]
    fn test_performance_metrics() {
        let context = DynamicContext::new_wasm().unwrap();
        let performance = context.get_performance().unwrap();
        assert!(performance.now() > 0.0);
    }
}
