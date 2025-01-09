/**
 * A dynamic JavaScript runtime powered by the Boa engine
 */
export class DynamicRuntime {
    /**
     * Create a new dynamic runtime instance
     */
    constructor();

    /**
     * Register a module with the runtime
     * @param name The module name/specifier
     * @param source The module's source code
     * @throws {Error} If module registration fails
     */
    register_module(name: string, source: string): void;

    /**
     * Register a JavaScript object as a module
     * @param moduleName The name of the module
     * @param exports An object containing the module's exports
     * @throws {Error} If module registration fails
     */
    register_module_exports(moduleName: string, exports: Record<string, any>): void;

    /**
     * Register a JavaScript function as a native function in a module
     * @param moduleName The module to add the function to
     * @param functionName The name of the function
     * @param func The JavaScript function to register
     * @throws {Error} If function registration fails
     */
    register_native_function(
        moduleName: string,
        functionName: string,
        func: Function
    ): void;

    /**
     * Load a module by its specifier
     * @param specifier The module specifier
     * @returns A promise that resolves when the module is loaded
     * @throws {Error} If module loading fails
     */
    load_module(specifier: string): Promise<void>;

    /**
     * Evaluate JavaScript code with access to registered modules
     * @param code The JavaScript code to evaluate
     * @returns The result of evaluation
     * @throws {Error} If evaluation fails
     */
    evaluate(code: string): any;

    /**
     * Create a new value that can be used in the runtime
     * @param value The JavaScript value to convert
     * @returns A dynamic value
     * @throws {Error} If value conversion fails
     */
    create_value(value: any): any;
}

/**
 * Error types that can be thrown by the runtime
 */
export interface DynamicError extends Error {
    /** Error code indicating the type of error */
    code: 
        | 'MODULE_NOT_FOUND'
        | 'CIRCULAR_DEPENDENCY'
        | 'TYPE_ERROR'
        | 'EXECUTION_ERROR'
        | 'UNKNOWN_ERROR';
    
    /** Additional error details */
    details?: {
        /** The module name for module-related errors */
        module?: string;
        /** The dependency path for circular dependency errors */
        path?: string;
    };
}

/**
 * Example usage:
 * ```typescript
 * import { DynamicRuntime } from '@boa-dev/dynamic';
 * 
 * const runtime = new DynamicRuntime();
 * 
 * // Register a module
 * runtime.register_module('my-module', `
 *     export function hello() {
 *         return 'Hello from my module!';
 *     }
 * `);
 * 
 * // Register exports
 * runtime.register_module_exports('native-module', {
 *     greet: (name: string) => `Hello, ${name}!`,
 *     calculate: (a: number, b: number) => a + b
 * });
 * 
 * // Use modules
 * const result = runtime.evaluate(`
 *     import { hello } from 'my-module';
 *     import { greet, calculate } from 'native-module';
 *     
 *     console.log(hello());
 *     console.log(greet('World'));
 *     console.log(calculate(2, 3));
 * `);
 * ```
 */
