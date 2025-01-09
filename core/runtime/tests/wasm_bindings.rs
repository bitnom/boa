use boa_engine::{Context, JsValue};
use boa_runtime::bindings::WasmRuntimeBindings;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use js_sys::{Function, Object};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_wasm_module_registration() {
    let mut context = Context::default();
    let mut runtime = WasmRuntimeBindings::new();

    // Register a module
    runtime
        .register_module(
            "test",
            "export const value = 42; export function double(x) { return x * 2; }",
        )
        .unwrap();

    // Load and test module
    let result = runtime.load_module("test", &mut context).unwrap();
    let module_obj = result.as_object().unwrap();

    let value = module_obj.get("value", &mut context).unwrap();
    assert_eq!(value, JsValue::new(42));

    let double = module_obj.get("double", &mut context).unwrap();
    let result = double
        .as_object()
        .unwrap()
        .call(
            &JsValue::undefined(),
            &[JsValue::new(5)],
            &mut context,
        )
        .unwrap();
    assert_eq!(result, JsValue::new(10));
}

#[wasm_bindgen_test]
fn test_wasm_native_functions() {
    let mut context = Context::default();
    let mut runtime = WasmRuntimeBindings::new();

    // Register module
    runtime.register_module("math", "").unwrap();

    // Create native function
    let multiply = Function::new_with_args("a, b", "return a * b");

    // Add native function to module
    runtime
        .register_native_function("math", "multiply", &multiply, &mut context)
        .unwrap();

    // Test native function
    let result = runtime.load_module("math", &mut context).unwrap();
    let module_obj = result.as_object().unwrap();
    let multiply = module_obj.get("multiply", &mut context).unwrap();
    let result = multiply
        .as_object()
        .unwrap()
        .call(
            &JsValue::undefined(),
            &[JsValue::new(6), JsValue::new(7)],
            &mut context,
        )
        .unwrap();
    assert_eq!(result, JsValue::new(42));
}

#[wasm_bindgen_test]
fn test_wasm_module_resolver() {
    let mut context = Context::default();
    let mut runtime = WasmRuntimeBindings::new();

    // Create resolver function
    let resolver = Function::new_with_args(
        "specifier",
        "return specifier.startsWith('@/') ? '/src/' + specifier.slice(2) : null",
    );

    // Set resolver
    runtime.set_module_resolver(&resolver);

    // Register module that will be resolved
    runtime
        .register_module("/src/test.js", "export const value = 'resolved';")
        .unwrap();

    // Test resolver
    let result = runtime.load_module("@/test.js", &mut context).unwrap();
    let module_obj = result.as_object().unwrap();
    let value = module_obj.get("value", &mut context).unwrap();
    assert_eq!(value, JsValue::from("resolved"));
}

#[wasm_bindgen_test]
fn test_wasm_module_transformer() {
    let mut context = Context::default();
    let mut runtime = WasmRuntimeBindings::new();

    // Create transformer function
    let transformer = Function::new_with_args(
        "content",
        "return content.replace('\\'hello\\'', '\\'HELLO\\'')",
    );

    // Set transformer
    runtime.set_module_transformer(&transformer);

    // Register module
    runtime
        .register_module("test", "export const value = 'hello';")
        .unwrap();

    // Test transformer
    let result = runtime.load_module("test", &mut context).unwrap();
    let module_obj = result.as_object().unwrap();
    let value = module_obj.get("value", &mut context).unwrap();
    assert_eq!(value, JsValue::from("HELLO"));
}

#[wasm_bindgen_test]
fn test_wasm_global_objects() {
    let mut context = Context::default();
    let mut runtime = WasmRuntimeBindings::new();

    // Create global object
    let config = Object::new();
    js_sys::Reflect::set(&config, &"version".into(), &"1.0.0".into()).unwrap();
    js_sys::Reflect::set(&config, &"debug".into(), &true.into()).unwrap();

    // Add global object
    runtime
        .add_global_object("CONFIG", &config, &mut context)
        .unwrap();

    // Initialize globals
    runtime.init_globals(&mut context).unwrap();

    // Test global object
    let global = context.global_object();
    let config = global.get("CONFIG", &mut context).unwrap();
    let config_obj = config.as_object().unwrap();
    
    let version = config_obj.get("version", &mut context).unwrap();
    assert_eq!(version, JsValue::from("1.0.0"));
    
    let debug = config_obj.get("debug", &mut context).unwrap();
    assert_eq!(debug, JsValue::new(true));
}

#[wasm_bindgen_test]
fn test_wasm_filesystem_mounting() {
    let mut context = Context::default();
    let mut runtime = WasmRuntimeBindings::new();

    // Mount virtual path
    runtime.mount_fs("/modules", "./test_modules").unwrap();

    // This is a limited test since we can't easily create files in WASM environment
    // The actual filesystem operations would need to be tested in Node.js or browser environment
    let result = runtime.load_module("/modules/test.js", &mut context);
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
fn test_wasm_error_handling() {
    let mut context = Context::default();
    let runtime = WasmRuntimeBindings::new();

    // Test loading non-existent module
    let result = runtime.load_module("nonexistent", &mut context);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), JsValue::undefined());

    // Test invalid module syntax
    let mut runtime = WasmRuntimeBindings::new();
    runtime
        .register_module("invalid", "export const = 'invalid syntax';")
        .unwrap();

    let result = runtime.load_module("invalid", &mut context);
    assert!(result.is_err());
}
