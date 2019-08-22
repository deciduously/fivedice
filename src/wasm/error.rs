// error.rs contains the error type for the application
use std::fmt;
use wasm_bindgen::JsValue;
use widget_grid::window::WindowError;

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
