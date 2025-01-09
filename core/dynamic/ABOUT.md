# About Boa Dynamic

Boa Dynamic is a crate that extends the Boa JavaScript engine with dynamic runtime interop capabilities. It provides functionality for:

- Loading JavaScript modules at runtime
- Registering native Rust functions dynamically
- Converting between JavaScript and Rust values
- Managing a module registry
- Supporting asynchronous module loading

## Example Usage

```rust
use boa_dynamic::{DynamicContext, DynamicModule};

// Create a new dynamic context
let mut context = DynamicContext::new();

// Create a module
let mut module = DynamicModule::new(
    "my-module",
    r#"
    export function hello() {
        return "Hello from dynamic module!";
    }
    "#,
);

// Register the module
context.register_module("my-module", module)?;

// Use the module
let result = context.evaluate(r#"
    import { hello } from 'my-module';
    hello();
"#)?;

println!("{}", result); // "Hello from dynamic module!"
```

## Features

- **Dynamic Module Loading**: Load JavaScript modules at runtime
- **Native Function Integration**: Register Rust functions as JavaScript functions
- **Safe Type Conversion**: Convert between JavaScript and Rust types safely
- **Async Support**: Load modules asynchronously
- **Module Registry**: Manage and track loaded modules
- **WASM Compatible**: Works in both native and WASM environments
