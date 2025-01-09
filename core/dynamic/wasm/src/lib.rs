use boa_dynamic::{DynamicContext, DynamicModule, Error};
use boa_engine::JsValue;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

/// Initialize panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
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
        self.context
            .register_module(name, module)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Load a module by its specifier
    #[wasm_bindgen]
    pub fn load_module(&mut self, specifier: &str) -> js_sys::Promise {
        let mut context = self.context.clone();
        future_to_promise(async move {
            context
                .load_module(specifier)
                .await
                .map(|_| JsValue::undefined())
                .map_err(|e| JsError::new(&e.to_string()).into())
        })
    }

    /// Evaluate JavaScript code
    #[wasm_bindgen]
    pub fn evaluate(&mut self, code: &str) -> Result<JsValue, JsError> {
        self.context
            .evaluate(code)
            .map(Into::into)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Register a native function
    #[wasm_bindgen]
    pub fn register_native_function(
        &mut self,
        module_name: &str,
        function_name: &str,
        function: &js_sys::Function,
    ) -> Result<(), JsError> {
        // Convert JS function to a native function
        let js_function = function.clone();
        let native_function = move |_this: &JsValue, args: &[JsValue], _ctx: &mut boa_engine::Context| {
            let this = JsValue::undefined();
            let args = args.iter().cloned().map(Into::into).collect::<js_sys::Array>();
            let result = js_function.apply(&this, &args).unwrap();
            Ok(result.into())
        };

        // Create or get the module
        let mut module = DynamicModule::new(module_name, "");
        module.add_native_function(function_name, native_function.into());

        // Register the module
        self.context
            .register_module(module_name, module)
            .map_err(|e| JsError::new(&e.to_string()))
    }
}
