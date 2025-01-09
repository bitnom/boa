use boa_dynamic::Error as DynamicError;
use wasm_bindgen::JsError;
use js_sys::{Error as JsSystemError, Object};

/// Convert between Rust and JavaScript errors
pub trait ErrorConversion {
    /// Convert to a JavaScript error
    fn to_js_error(&self) -> JsError;
    
    /// Create a detailed error object
    fn to_detailed_error(&self) -> Object;
}

impl ErrorConversion for DynamicError {
    fn to_js_error(&self) -> JsError {
        JsError::new(&self.to_string())
    }
    
    fn to_detailed_error(&self) -> Object {
        let error = JsSystemError::new(&self.to_string());
        let obj = Object::from(error);
        
        // Add additional error details based on the error type
        match self {
            DynamicError::ModuleNotFound(module) => {
                let _ = js_sys::Reflect::set(
                    &obj,
                    &"module".into(),
                    &module.into(),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &"code".into(),
                    &"MODULE_NOT_FOUND".into(),
                );
            }
            DynamicError::CircularDependency(path) => {
                let _ = js_sys::Reflect::set(
                    &obj,
                    &"path".into(),
                    &path.into(),
                );
                let _ = js_sys::Reflect::set(
                    &obj,
                    &"code".into(),
                    &"CIRCULAR_DEPENDENCY".into(),
                );
            }
            _ => {
                let _ = js_sys::Reflect::set(
                    &obj,
                    &"code".into(),
                    &"UNKNOWN_ERROR".into(),
                );
            }
        }
        
        obj
    }
}
