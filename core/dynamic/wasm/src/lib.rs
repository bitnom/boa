use boa_dynamic::{DynamicContext, DynamicModule, DynamicValue, Error};
use boa_engine::JsValue;
use js_sys::{Array, Function, Object, Promise, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use std::sync::Arc;

/// Initialize panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Convert Rust errors to JavaScript errors
impl From<Error> for JsError {
    fn from(err: Error) -> Self {
        JsError::new(&err.to_string())
    }
}

/// The WASM runtime for Boa's dynamic capabilities
#[wasm_bindgen]
pub struct DynamicRuntime {
    context: DynamicContext,
}

#[wasm_bindgen]
impl DynamicRuntime {
    /// Create a new dynamic runtime
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            context: DynamicContext::new(),
        }
    }

    /// Register a module with the runtime
    #[wasm_bindgen]
    pub fn register_module(&mut self, name: &str, source: &str) -> Result<(), JsError> {
        let module = DynamicModule::new(name, source);
        self.context.register_module(name, module).map_err(Into::into)
    }

    /// Load a module by its specifier
    #[wasm_bindgen]
    pub fn load_module(&mut self, specifier: &str) -> Promise {
        let mut context = self.context.clone();
        future_to_promise(async move {
            context
                .load_module(specifier)
                .await
                .map(|_| JsValue::undefined())
                .map_err(|e| JsError::from(e).into())
        })
    }

    /// Evaluate JavaScript code
    #[wasm_bindgen]
    pub fn evaluate(&mut self, code: &str) -> Result<JsValue, JsError> {
        self.context
            .evaluate(code)
            .map(Into::into)
            .map_err(Into::into)
    }

    /// Register a native function
    #[wasm_bindgen]
    pub fn register_native_function(
        &mut self,
        module_name: &str,
        function_name: &str,
        function: &Function,
    ) -> Result<(), JsError> {
        let js_function = function.clone();
        
        // Create a native function that wraps the JS function
        let native_function = move |_this: &JsValue, args: &[JsValue], _ctx: &mut boa_engine::Context| {
            let this = JsValue::undefined();
            let js_args = args.iter().cloned().map(Into::into).collect::<Array>();
            
            match js_function.apply(&this, &js_args) {
                Ok(result) => Ok(result.into()),
                Err(e) => Err(boa_engine::JsError::from_opaque(e.into())),
            }
        };

        // Create or get the module
        let mut module = DynamicModule::new(module_name, "");
        module.add_native_function(function_name, native_function.into());

        // Register the module
        self.context.register_module(module_name, module).map_err(Into::into)
    }

    /// Register a module with exports object
    #[wasm_bindgen]
    pub fn register_module_exports(
        &mut self,
        module_name: &str,
        exports: &Object,
    ) -> Result<(), JsError> {
        let mut module = DynamicModule::new(module_name, "");
        
        // Get all enumerable properties of the exports object
        let keys = Object::keys(exports);
        let len = keys.length();
        
        for i in 0..len {
            let key = keys.get(i);
            let key_str = key.as_string().ok_or_else(|| {
                JsError::new("Export key must be a string")
            })?;
            
            let value = Reflect::get(exports, &key).map_err(|e| {
                JsError::new(&format!("Failed to get export value: {}", e))
            })?;
            
            // If the export is a function, register it as a native function
            if Function::instanceof(&value) {
                let function: Function = value.dyn_into().unwrap();
                let js_function = function.clone();
                
                let native_function = move |_this: &JsValue, args: &[JsValue], _ctx: &mut boa_engine::Context| {
                    let this = JsValue::undefined();
                    let js_args = args.iter().cloned().map(Into::into).collect::<Array>();
                    
                    match js_function.apply(&this, &js_args) {
                        Ok(result) => Ok(result.into()),
                        Err(e) => Err(boa_engine::JsError::from_opaque(e.into())),
                    }
                };
                
                module.add_native_function(&key_str, native_function.into());
            }
        }
        
        self.context.register_module(module_name, module).map_err(Into::into)
    }

    /// Create a new value from a JavaScript value
    #[wasm_bindgen]
    pub fn create_value(&mut self, value: &JsValue) -> Result<DynamicValue, JsError> {
        Ok(DynamicValue::new(value.clone().into()))
    }
}

impl Default for DynamicRuntime {
    fn default() -> Self {
        Self::new()
    }
}
