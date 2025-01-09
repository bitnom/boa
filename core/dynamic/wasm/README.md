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

// Register a module
await runtime.registerModule("my-module", `
  export function hello() {
    return "Hello from dynamic module!";
  }
`);

// Register a native JavaScript function
runtime.registerNativeFunction("my-module", "nativeHello", () => {
  return "Hello from native function!";
});

// Use the module
const result = runtime.evaluate(`
  import { hello, nativeHello } from 'my-module';
  console.log(hello());        // "Hello from dynamic module!"
  console.log(nativeHello());  // "Hello from native function!"
`);
```

## API Reference

### `DynamicRuntime`

The main class for interacting with Boa's dynamic capabilities.

#### Constructor

- `new DynamicRuntime()`: Create a new dynamic runtime instance

#### Methods

- `registerModule(name: string, source: string): Promise<void>`
  Register a JavaScript module with the runtime

- `loadModule(specifier: string): Promise<void>`
  Load a previously registered module

- `evaluate(code: string): any`
  Evaluate JavaScript code with access to registered modules

- `registerNativeFunction(moduleName: string, functionName: string, function: Function): void`
  Register a JavaScript function as a native function in a module

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
