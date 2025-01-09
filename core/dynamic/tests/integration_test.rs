use boa_dynamic::{DynamicContext, Error, Result};
use tokio;

#[tokio::test]
async fn test_module_loading() -> Result<()> {
    let mut context = DynamicContext::new();

    // Register a module
    context.register_module("math", r#"
        export function add(a, b) {
            return a + b;
        }

        export function multiply(a, b) {
            return a * b;
        }
    "#)?;

    // Use the module
    let result = context.evaluate(r#"
        import { add, multiply } from 'math';
        const sum = add(2, 3);
        const product = multiply(4, 5);
        [sum, product]
    "#)?;

    assert!(result.is_array());
    // TODO: Add proper array assertions once we have array support

    Ok(())
}

#[tokio::test]
async fn test_circular_dependencies() -> Result<()> {
    let mut context = DynamicContext::new();

    // Register modules with circular dependency
    context.register_module("a", r#"
        import { value } from 'b';
        export const value = 1;
    "#)?;

    context.register_module("b", r#"
        import { value } from 'a';
        export const value = 2;
    "#)?;

    // Attempt to load module should fail with circular dependency error
    let result = context.evaluate(r#"
        import { value } from 'a';
        value
    "#);

    assert!(matches!(result, Err(Error::CircularDependency { .. })));

    Ok(())
}

#[tokio::test]
async fn test_module_caching() -> Result<()> {
    let mut context = DynamicContext::new();

    // Register a module that tracks instantiation
    context.register_module("counter", r#"
        let count = 0;
        export function increment() {
            return ++count;
        }
    "#)?;

    // First load should initialize count to 0
    let result1 = context.evaluate(r#"
        import { increment } from 'counter';
        increment()
    "#)?;

    // Second load should use cached module
    let result2 = context.evaluate(r#"
        import { increment } from 'counter';
        increment()
    "#)?;

    assert!(result1.is_number());
    assert!(result2.is_number());
    // TODO: Add proper number assertions once we have number support

    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let mut context = DynamicContext::new();

    // Test syntax error
    let result = context.evaluate("invalid javascript;");
    assert!(matches!(result, Err(Error::Execution { .. })));

    // Test module not found
    let result = context.evaluate(r#"
        import { something } from 'non-existent';
    "#);
    assert!(matches!(result, Err(Error::ModuleNotFound { .. })));

    // Test invalid module name
    let result = context.register_module("invalid-name!", "");
    assert!(matches!(result, Err(Error::InvalidModuleName { .. })));

    Ok(())
}

#[cfg(feature = "wasm")]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_wasm_module_loading() {
        let mut context = DynamicContext::new_wasm().unwrap();
        
        context.register_module("math", r#"
            export function add(a, b) {
                return a + b;
            }
        "#).unwrap();

        let result = context.evaluate_async(r#"
            import { add } from 'math';
            add(2, 3)
        "#).unwrap();

        assert!(result.is_promise());
    }
}
