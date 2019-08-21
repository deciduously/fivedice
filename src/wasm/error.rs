// error.rs contains the error type for the application

use std::fmt;
use wasm_bindgen::JsValue;

/// All possible Error types
#[derive(Debug)]
pub enum FiveDiceError {
    Canvas(String),
    Interop(String),
    OutOfBounds,
}

impl fmt::Display for FiveDiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Canvas(typename) => write!(f, "Could not write {} to the canvas!", typename),
            Self::Interop(detail) => write!(f, "Interop error: {}", detail),
            Self::OutOfBounds => write!(f, "Attempted to scroll cursor out of bounds!")
        }
    }
}

pub type Result<T> = std::result::Result<T, FiveDiceError>;

impl std::error::Error for FiveDiceError {}

impl Into<JsValue> for FiveDiceError {
    fn into(self) -> JsValue {
        format!("{}", self).into()
    }
}
