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
- **Module Resolution**: Support for relative and absolute imports
- **WASM Compatible**: Works in both native and WASM environments

## Module Resolution

The module resolution system supports both relative and absolute imports:

```rust
use boa_dynamic::{DynamicContext, DynamicModule};

let mut context = DynamicContext::new();

// Add base paths for module resolution
context.add_base_path("/path/to/modules");

// Register modules
context.register_module("math", math_module)?;
context.register_module("./utils/helpers", helpers_module)?;

// Import modules (supports both absolute and relative paths)
let math = context.import("math").await?;           // Absolute import
let helpers = context.import("./utils/helpers").await?;  // Relative import

// Use relative imports in JavaScript code
let result = context.evaluate(r#"
    import { add } from 'math';
    import { format } from './utils/helpers';
    
    format(add(1, 2));
"#)?;
```

### Base Paths

You can add multiple base paths for module resolution. When resolving a module, the system will:
1. For relative imports (starting with './' or '../'), resolve relative to the importing module
2. For absolute imports, search in each base path in order
3. If not found in base paths, treat as a module name
