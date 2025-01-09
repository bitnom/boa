use boa_dynamic::DynamicValue;
use boa_engine::JsValue;
use js_sys::{Array, Object, Function};
use wasm_bindgen::JsError;
use std::convert::TryFrom;

/// Trait for converting between JavaScript and Rust values
pub trait ValueConversion {
    /// Convert to a JavaScript value
    fn to_js_value(&self) -> Result<JsValue, JsError>;
    
    /// Convert from a JavaScript value
    fn from_js_value(value: &JsValue) -> Result<Self, JsError>
    where
        Self: Sized;
}

impl ValueConversion for DynamicValue {
    fn to_js_value(&self) -> Result<JsValue, JsError> {
        Ok(self.clone().into_inner().into())
    }
    
    fn from_js_value(value: &JsValue) -> Result<Self, JsError> {
        Ok(Self::new(value.clone().into()))
    }
}

/// Convert a JavaScript array to a Vec of DynamicValues
pub fn array_to_vec(array: &Array) -> Result<Vec<DynamicValue>, JsError> {
    let mut result = Vec::with_capacity(array.length() as usize);
    for i in 0..array.length() {
        let value = array.get(i);
        result.push(DynamicValue::from_js_value(&value)?);
    }
    Ok(result)
}

/// Convert a Vec of DynamicValues to a JavaScript array
pub fn vec_to_array(vec: &[DynamicValue]) -> Result<Array, JsError> {
    let array = Array::new_with_length(vec.len() as u32);
    for (i, value) in vec.iter().enumerate() {
        array.set(i as u32, value.to_js_value()?);
    }
    Ok(array)
}

/// Convert a JavaScript object to a HashMap of DynamicValues
pub fn object_to_map(obj: &Object) -> Result<std::collections::HashMap<String, DynamicValue>, JsError> {
    let mut result = std::collections::HashMap::new();
    let keys = Object::keys(obj);
    for i in 0..keys.length() {
        let key = keys.get(i);
        let key_str = key.as_string().ok_or_else(|| {
            JsError::new("Object key must be a string")
        })?;
        let value = js_sys::Reflect::get(obj, &key).map_err(|e| {
            JsError::new(&format!("Failed to get object value: {}", e))
        })?;
        result.insert(key_str, DynamicValue::from_js_value(&value)?);
    }
    Ok(result)
}
