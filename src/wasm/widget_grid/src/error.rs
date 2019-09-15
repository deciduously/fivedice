use crate::types::Point;
use std::fmt;
use wasm_bindgen::JsValue;

/// Window error type
#[derive(Debug)]
pub enum WindowError {
    DomError(String),
    Element,
    JsVal(JsValue),
    OutOfBounds(Point, Point),
    Text,
}

impl fmt::Display for WindowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::DomError(s) => write!(f, "DOM problem: {}", s),
            Self::Element => write!(f, "Could not append element to DOM"),
            Self::JsVal(js) => write!(f, "{:#?}", js),
            Self::Text => write!(f, "Could not add text to the window"),
            Self::OutOfBounds(origin, destination) => write!(
                f,
                "Attempted to scroll cursor out of bounds from {} to {}",
                origin, destination
            ),
        }
    }
}

impl From<JsValue> for WindowError {
    fn from(other: JsValue) -> Self {
        WindowError::JsVal(other)
    }
}

impl Into<JsValue> for WindowError {
    fn into(self) -> JsValue {
        format!("{}", self).into()
    }
}

impl std::error::Error for WindowError {}

pub type Result<T> = std::result::Result<T, WindowError>;
