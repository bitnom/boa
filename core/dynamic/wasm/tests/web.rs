use wasm_bindgen_test::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys::{Function, Object};
use boa_dynamic_wasm::DynamicRuntime;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_runtime_creation() {
    let runtime = DynamicRuntime::new();
    assert!(runtime.evaluate("2 + 2").unwrap().as_f64().unwrap() == 4.0);
}

#[wasm_bindgen_test]
async fn test_module_registration() {
    let mut runtime = DynamicRuntime::new();
    
    let result = runtime.register_module(
        "test-module",
        r#"
        export function hello() {
            return "Hello from WASM!";
        }
        "#,
    );
    assert!(result.is_ok());

    let eval_result = runtime.evaluate(r#"
        import { hello } from 'test-module';
        hello();
    "#).unwrap();
    
    assert_eq!(eval_result.as_string().unwrap(), "Hello from WASM!");
}

#[wasm_bindgen_test]
async fn test_native_function_registration() {
    let mut runtime = DynamicRuntime::new();

    // Create a JavaScript function
    let js_function = Function::new_with_args("", "return 'Hello from JS!';");

    // Register it as a native function
    runtime.register_native_function("native-module", "nativeHello", &js_function).unwrap();

    // Evaluate the function
    let result = runtime.evaluate("nativeHello();").unwrap();
    assert_eq!(result.as_string().unwrap(), "Hello from JS!");
}

#[wasm_bindgen_test]
async fn test_module_loading() {
    let mut runtime = DynamicRuntime::new();

    // Register a module
    runtime.register_module(
        "async-module",
        r#"
        export async function delayed() {
            return "Delayed response";
        }
        "#,
    ).unwrap();

    // Load the module
    let promise = runtime.load_module("async-module");
    let result = JsFuture::from(promise).await;
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
async fn test_error_handling() {
    let mut runtime = DynamicRuntime::new();

    // Test invalid JavaScript
    let result = runtime.evaluate("invalid javascript;");
    assert!(result.is_err());

    // Test loading non-existent module
    let promise = runtime.load_module("non-existent");
    let result = JsFuture::from(promise).await;
    assert!(result.is_err());
}

#[wasm_bindgen_test]
async fn test_complex_interaction() {
    let mut runtime = DynamicRuntime::new();

    // Register a module with both JS and native functions
    runtime.register_module(
        "complex-module",
        r#"
        export function multiply(x) {
            return x * getNativeValue();
        }
        "#,
    ).unwrap();

    // Register a native function that provides a value
    let get_value = Function::new_with_args("", "return 42;");
    runtime.register_native_function("complex-module", "getNativeValue", &get_value).unwrap();

    // Test the interaction
    let result = runtime.evaluate(r#"
        import { multiply } from 'complex-module';
        multiply(2);
    "#).unwrap();

    assert_eq!(result.as_f64().unwrap(), 84.0);
}
