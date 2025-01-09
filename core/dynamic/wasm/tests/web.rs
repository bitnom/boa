use wasm_bindgen_test::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys::{Function, Object, Reflect};
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
async fn test_module_exports() {
    let mut runtime = DynamicRuntime::new();
    
    // Create a JavaScript object with exports
    let exports = Object::new();
    let hello_fn = Function::new_with_args("", "return 'Hello from exports!';");
    assert!(Reflect::set(&exports, &"hello".into(), &hello_fn).is_ok());
    
    // Register the exports as a module
    runtime.register_module_exports("exports-module", &exports).unwrap();
    
    // Test using the exported function
    let result = runtime.evaluate(r#"
        import { hello } from 'exports-module';
        hello();
    "#).unwrap();
    
    assert_eq!(result.as_string().unwrap(), "Hello from exports!");
}

#[wasm_bindgen_test]
async fn test_circular_dependency() {
    let mut runtime = DynamicRuntime::new();
    
    // Register two modules with circular dependency
    runtime.register_module(
        "module-a",
        r#"
        import { b } from 'module-b';
        export function a() {
            return b();
        }
        "#,
    ).unwrap();
    
    runtime.register_module(
        "module-b",
        r#"
        import { a } from 'module-a';
        export function b() {
            return a();
        }
        "#,
    ).unwrap();
    
    // Attempting to use these modules should result in a circular dependency error
    let result = runtime.evaluate(r#"
        import { a } from 'module-a';
        a();
    "#);
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("circular"));
}

#[wasm_bindgen_test]
async fn test_native_function_error_handling() {
    let mut runtime = DynamicRuntime::new();
    
    // Create a JavaScript function that throws
    let error_fn = Function::new_with_args("", "throw new Error('Test error');");
    runtime.register_native_function("error-module", "throwError", &error_fn).unwrap();
    
    // Test that the error is properly propagated
    let result = runtime.evaluate("throwError();");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Test error"));
}

#[wasm_bindgen_test]
async fn test_value_conversion() {
    let mut runtime = DynamicRuntime::new();
    
    // Test object conversion
    let obj = Object::new();
    assert!(Reflect::set(&obj, &"key".into(), &"value".into()).is_ok());
    
    let value = runtime.create_value(&obj.into()).unwrap();
    
    // Test using the converted value
    runtime.register_module(
        "value-module",
        r#"
        export function checkValue(val) {
            return val.key === 'value';
        }
        "#,
    ).unwrap();
    
    let result = runtime.evaluate(r#"
        import { checkValue } from 'value-module';
        checkValue(value);
    "#).unwrap();
    
    assert!(result.as_bool().unwrap());
}

#[wasm_bindgen_test]
async fn test_async_module_loading() {
    let mut runtime = DynamicRuntime::new();
    
    // Register an async module
    runtime.register_module(
        "async-module",
        r#"
        export async function delayed() {
            return new Promise(resolve => 
                setTimeout(() => resolve('delayed result'), 100)
            );
        }
        "#,
    ).unwrap();
    
    // Load the module
    let promise = runtime.load_module("async-module");
    let result = JsFuture::from(promise).await;
    assert!(result.is_ok());
    
    // Test using the async function
    let eval_result = runtime.evaluate(r#"
        import { delayed } from 'async-module';
        delayed();
    "#).unwrap();
    
    let promise = js_sys::Promise::resolve(&eval_result);
    let result = JsFuture::from(promise).await.unwrap();
    assert_eq!(result.as_string().unwrap(), "delayed result");
}
