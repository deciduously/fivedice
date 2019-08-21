// error.rs contains the error type for the application
use crate::draw::WindowError;
use std::fmt;
use wasm_bindgen::JsValue;

/// All possible Error types
#[derive(Debug)]
pub enum FiveDiceError {
    Window(WindowError),
    Interop(JsValue),
}

impl fmt::Display for FiveDiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Window(e) => write!(f, "{}", e),
            Self::Interop(js) => write!(f, "{:#?}", js),
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

impl From<WindowError> for FiveDiceError {
    fn from(e: WindowError) -> Self {
        FiveDiceError::Window(e)
    }
}
