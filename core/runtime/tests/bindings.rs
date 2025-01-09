use boa_engine::{Context, JsValue, NativeFunction, Object};
use boa_runtime::bindings::{ModuleDefinition, RuntimeBindings};
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_module_registration() {
    let mut context = Context::default();
    let mut runtime = RuntimeBindings::new();

    // Register a simple module
    let module = ModuleDefinition::new(
        "test",
        "export const value = 42; export function double(x) { return x * 2; }",
    );
    runtime.register_module(module);

    // Load and evaluate module
    let result = runtime.load_module("test", &mut context).unwrap();
    let module_obj = result.as_object().unwrap();

    // Check exports
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

#[test]
fn test_native_functions() {
    let mut context = Context::default();
    let mut runtime = RuntimeBindings::new();

    // Create module with native function
    let mut module = ModuleDefinition::new("math", "");
    let multiply = NativeFunction::from_fn_ptr(|_this, args, ctx| {
        let a = args.get_or_undefined(0).to_number(ctx)?;
        let b = args.get_or_undefined(1).to_number(ctx)?;
        Ok(JsValue::new(a * b))
    });
    module.add_native_function("multiply", multiply);
    runtime.register_module(module);

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

#[test]
fn test_filesystem_mounting() {
    let temp_dir = TempDir::new().unwrap();
    let module_path = temp_dir.path().join("test.js");
    std::fs::write(&module_path, "export const value = 'hello';").unwrap();

    let mut context = Context::default();
    let mut runtime = RuntimeBindings::new();

    // Mount temp directory
    runtime.mount_fs("/modules", temp_dir.path());

    // Load module from filesystem
    let result = runtime
        .load_module("/modules/test.js", &mut context)
        .unwrap();
    let module_obj = result.as_object().unwrap();
    let value = module_obj.get("value", &mut context).unwrap();
    assert_eq!(value, JsValue::from("hello"));
}

#[test]
fn test_module_resolver() {
    let mut context = Context::default();
    let mut runtime = RuntimeBindings::new();

    // Set module resolver
    runtime.set_module_resolver(|specifier| {
        if specifier.starts_with("@/") {
            Some(format!("/src/{}", &specifier[2..]))
        } else {
            None
        }
    });

    // Register a module that will be resolved
    let module = ModuleDefinition::new(
        "/src/test.js",
        "export const value = 'resolved';",
    );
    runtime.register_module(module);

    // Load module using alias
    let result = runtime.load_module("@/test.js", &mut context).unwrap();
    let module_obj = result.as_object().unwrap();
    let value = module_obj.get("value", &mut context).unwrap();
    assert_eq!(value, JsValue::from("resolved"));
}

#[test]
fn test_module_transformer() {
    let mut context = Context::default();
    let mut runtime = RuntimeBindings::new();

    // Set module transformer
    runtime.set_module_transformer(|content| {
        // Simple transformer that uppercases string literals
        content.replace("'hello'", "'HELLO'")
    });

    // Register module with content to transform
    let module = ModuleDefinition::new(
        "test",
        "export const value = 'hello';",
    );
    runtime.register_module(module);

    // Load and check transformed content
    let result = runtime.load_module("test", &mut context).unwrap();
    let module_obj = result.as_object().unwrap();
    let value = module_obj.get("value", &mut context).unwrap();
    assert_eq!(value, JsValue::from("HELLO"));
}

#[test]
fn test_global_objects() {
    let mut context = Context::default();
    let mut runtime = RuntimeBindings::new();

    // Create and add global object
    let mut config = Object::default();
    config
        .set("version", JsValue::new("1.0.0"), true, &mut context)
        .unwrap();
    config
        .set("debug", JsValue::new(true), true, &mut context)
        .unwrap();
    runtime.add_global_object("CONFIG", config);

    // Initialize globals
    runtime.init_globals(&mut context).unwrap();

    // Test accessing global object
    let global = context.global_object();
    let config = global.get("CONFIG", &mut context).unwrap();
    let config_obj = config.as_object().unwrap();
    
    let version = config_obj.get("version", &mut context).unwrap();
    assert_eq!(version, JsValue::from("1.0.0"));
    
    let debug = config_obj.get("debug", &mut context).unwrap();
    assert_eq!(debug, JsValue::new(true));
}

#[test]
fn test_module_error_handling() {
    let mut context = Context::default();
    let runtime = RuntimeBindings::new();

    // Try loading non-existent module
    let result = runtime.load_module("nonexistent", &mut context);
    assert!(result.is_ok()); // Should return undefined, not error
    assert_eq!(result.unwrap(), JsValue::undefined());

    // Try loading module with syntax error
    let mut runtime = RuntimeBindings::new();
    let module = ModuleDefinition::new(
        "invalid",
        "export const = 'invalid syntax';", // Invalid syntax
    );
    runtime.register_module(module);

    let result = runtime.load_module("invalid", &mut context);
    assert!(result.is_err());
}
