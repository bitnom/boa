use crate::Error;
use boa_engine::{Context, JsResult, JsValue};
use std::convert::TryFrom;

/// A wrapper around JsValue that provides additional conversion capabilities.
#[derive(Debug, Clone)]
pub struct DynamicValue(JsValue);

impl DynamicValue {
    /// Create a new dynamic value
    pub fn new(value: JsValue) -> Self {
        Self(value)
    }

    /// Try to convert this value to a Rust type
    pub fn try_into<T>(&self, context: &mut Context) -> Result<T, Error>
    where
        T: TryFrom<JsValue, Error = boa_engine::JsError>,
    {
        T::try_from(self.0.clone()).map_err(|e| Error::TypeConversion(e.to_string()))
    }

    /// Get the inner JsValue
    pub fn into_inner(self) -> JsValue {
        self.0
    }
}

impl From<JsValue> for DynamicValue {
    fn from(value: JsValue) -> Self {
        Self(value)
    }
}

impl From<DynamicValue> for JsValue {
    fn from(value: DynamicValue) -> Self {
        value.0
    }
}
