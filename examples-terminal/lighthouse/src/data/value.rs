//! Simple typed values used for flag payloads and comparisons.

use serde::{Deserialize, Serialize};

/// A typed value stored in a flag or compared against in a condition.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Value {
    /// A boolean flag value. The convention for simple "is this flag set"
    /// checks is `Value::Bool(true)`.
    Bool(bool),
    /// An integer value.
    Int(i64),
    /// A free-form text value.
    Text(String),
}

impl Value {
    pub const TRUE: Value = Value::Bool(true);
    pub const FALSE: Value = Value::Bool(false);

    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(inner) = self {
            Some(*inner)
        } else {
            None
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        if let Value::Int(inner) = self {
            Some(*inner)
        } else {
            None
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        if let Value::Text(inner) = self {
            Some(inner.as_str())
        } else {
            None
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Int(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Int(value as i64)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::Text(value.to_string())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Text(value)
    }
}
