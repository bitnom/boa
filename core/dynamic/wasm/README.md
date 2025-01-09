# Boa Dynamic WASM

WASM bindings for Boa's dynamic runtime interop capabilities.

## Installation

```bash
npm install @boa-dev/dynamic
```

## Usage

```javascript
import { DynamicRuntime } from "@boa-dev/dynamic";

// Create a new runtime
const runtime = new DynamicRuntime();

// Add base paths for module resolution
runtime.addBasePath("/path/to/modules");

// Register modules with relative paths
await runtime.registerModule("math", `
  export function add(a, b) {
    return a + b;
  }
`);

await runtime.registerModule("./utils/format", `
  export function format(num) {
    return \`Result: \${num}\`;
  }
`);

// Use modules with relative imports
const result = runtime.evaluate(`
  import { add } from 'math';
  import { format } from './utils/format';
  
  format(add(1, 2));  // "Result: 3"
`);
```

## API Reference

### `DynamicRuntime`

The main class for interacting with Boa's dynamic capabilities.

#### Constructor

- `new DynamicRuntime()`: Create a new dynamic runtime instance

#### Methods

- `addBasePath(path: string): void`: Add a base path for module resolution
- `registerModule(name: string, source: string): Promise<void>`: Register a module with the runtime
- `registerNativeFunction(moduleName: string, functionName: string, func: Function): void`: Register a JavaScript function as a native function
- `evaluate(code: string): any`: Evaluate JavaScript code with access to registered modules

### Module Resolution

The runtime supports both relative and absolute module imports:

- Relative imports start with `./` or `../` and are resolved relative to the importing module
- Absolute imports are resolved using the configured base paths
- If a module isn't found in base paths, it's treated as a module name

Example:
```javascript
// Register modules
await runtime.registerModule("math/operations", mathCode);
await runtime.registerModule("./utils/helpers", helpersCode);

// Use modules
const result = await runtime.evaluate(`
  import { add } from 'math/operations';  // Absolute import
  import { format } from './utils/helpers';  // Relative import
  
  format(add(1, 2));
`);
```

## Building from Source

1. Install wasm-pack:
```bash
cargo install wasm-pack
```

2. Build the package:
```bash
wasm-pack build --target web
```

3. The built package will be in the `pkg` directory
