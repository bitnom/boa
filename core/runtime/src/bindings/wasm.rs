use super::{ModuleDefinition, RuntimeBindings};
use boa_engine::{Context, JsResult, JsValue, NativeFunction, Object};
use wasm_bindgen::prelude::*;
use js_sys::{Function, Object as JsObject};
use std::path::PathBuf;

#[wasm_bindgen]
pub struct WasmRuntimeBindings {
    inner: RuntimeBindings,
}

#[wasm_bindgen]
impl WasmRuntimeBindings {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: RuntimeBindings::new(),
        }
    }

    /// Register a new module with JavaScript content
    #[wasm_bindgen]
    pub fn register_module(&mut self, name: &str, content: &str) -> Result<(), JsError> {
        let mut module = ModuleDefinition::new(name, content);
        self.inner.register_module(module);
        Ok(())
    }

    /// Register a native function in a module
    #[wasm_bindgen]
    pub fn register_native_function(
        &mut self,
        module_name: &str,
        function_name: &str,
        function: &Function,
        context: &mut Context,
    ) -> Result<(), JsError> {
        if let Some(module) = self.inner.get_module(module_name) {
            let native_fn = NativeFunction::from_js_function(function.clone(), context)?;
            module.add_native_function(function_name, native_fn);
            Ok(())
        } else {
            Err(JsError::new(&format!("Module '{}' not found", module_name)))
        }
    }

    /// Mount a filesystem path
    #[wasm_bindgen]
    pub fn mount_fs(&mut self, virtual_path: &str, real_path: &str) -> Result<(), JsError> {
        self.inner.mount_fs(virtual_path, PathBuf::from(real_path));
        Ok(())
    }

    /// Set module resolver
    #[wasm_bindgen]
    pub fn set_module_resolver(&mut self, resolver: &Function) {
        let resolver = resolver.clone();
        self.inner.set_module_resolver(move |specifier| {
            let this = JsValue::undefined();
            let result = resolver
                .call1(&this, &JsValue::from_str(specifier))
                .ok()?;
            if result.is_string() {
                Some(result.as_string().unwrap())
            } else {
                None
            }
        });
    }

    /// Set module transformer
    #[wasm_bindgen]
    pub fn set_module_transformer(&mut self, transformer: &Function) {
        let transformer = transformer.clone();
        self.inner.set_module_transformer(move |content| {
            let this = JsValue::undefined();
            let result = transformer
                .call1(&this, &JsValue::from_str(&content))
                .unwrap_or(JsValue::from_str(&content));
            result.as_string().unwrap_or(content)
        });
    }

    /// Add a global object
    #[wasm_bindgen]
    pub fn add_global_object(
        &mut self,
        name: &str,
        obj: &JsObject,
        context: &mut Context,
    ) -> Result<(), JsError> {
        let obj = Object::from_js_object(obj.clone(), context)?;
        self.inner.add_global_object(name, obj);
        Ok(())
    }

    /// Initialize global objects
    #[wasm_bindgen]
    pub fn init_globals(&self, context: &mut Context) -> Result<(), JsError> {
        self.inner
            .init_globals(context)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Load and evaluate a module
    #[wasm_bindgen]
    pub fn load_module(&self, name: &str, context: &mut Context) -> Result<JsValue, JsError> {
        self.inner
            .load_module(name, context)
            .map_err(|e| JsError::new(&e.to_string()))
    }
}
