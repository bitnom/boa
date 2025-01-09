use crate::{DynamicContext, DynamicModule, Error};
use boa_engine::{Context, JsValue, NativeFunction};

#[tokio::test]
async fn test_module_registration() {
    let mut context = DynamicContext::new();
    let module = DynamicModule::new(
        "test-module",
        r#"
        export function hello() {
            return "Hello from module!";
        }
        "#,
    );

    assert!(context.register_module("test-module", module).is_ok());
    assert!(context.evaluate(r#"
        import { hello } from 'test-module';
        hello();
    "#).is_ok());
}

#[tokio::test]
async fn test_native_function() {
    let mut context = DynamicContext::new();
    
    // Create a module with a native function
    let mut module = DynamicModule::new("native-module", "");
    let native_fn = NativeFunction::from_fn_ptr(|_this, _args, _ctx| {
        Ok(JsValue::from("Hello from native function!"))
    });
    module.add_native_function("nativeHello", native_fn);

    // Register and test the module
    assert!(context.register_module("native-module", module).is_ok());
    let result = context.evaluate(r#"nativeHello();"#).unwrap();
    assert_eq!(
        result.to_string(&mut context.inner_mut()).unwrap().to_string(),
        "\"Hello from native function!\""
    );
}

#[tokio::test]
async fn test_module_loading() {
    let mut context = DynamicContext::new();
    
    // Register a module
    let module = DynamicModule::new(
        "async-module",
        r#"
        export async function delayed() {
            return "Delayed response";
        }
        "#,
    );
    context.register_module("async-module", module).unwrap();

    // Load the module
    let loaded_module = context.load_module("async-module").await;
    assert!(loaded_module.is_ok());
}

#[tokio::test]
async fn test_error_handling() {
    let mut context = DynamicContext::new();

    // Test invalid JavaScript
    let result = context.evaluate("invalid javascript;");
    assert!(matches!(result, Err(Error::Execution(_))));

    // Test loading non-existent module
    let result = context.load_module("non-existent").await;
    assert!(matches!(result, Err(Error::ModuleLoading(_))));
}

#[tokio::test]
async fn test_module_interaction() {
    let mut context = DynamicContext::new();

    // Register two modules that interact
    let module1 = DynamicModule::new(
        "module1",
        r#"
        export function getValue() {
            return 42;
        }
        "#,
    );
    let module2 = DynamicModule::new(
        "module2",
        r#"
        import { getValue } from 'module1';
        export function double() {
            return getValue() * 2;
        }
        "#,
    );

    context.register_module("module1", module1).unwrap();
    context.register_module("module2", module2).unwrap();

    let result = context.evaluate(r#"
        import { double } from 'module2';
        double();
    "#).unwrap();

    assert_eq!(
        result.to_string(&mut context.inner_mut()).unwrap().to_string(),
        "84"
    );
}
