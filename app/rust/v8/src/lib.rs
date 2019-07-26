extern crate libc;

mod sys {
    pub mod util;
    #[macro_use]
    pub mod macros;
    pub use macros::*;

    pub mod fun;
    /// V8 types
    pub mod types;
}

mod macros;
pub use macros::*;
pub use sys::*;

use crate::sys::types::{Handle, JsArray, JsArrayBuffer, JsNumber, JsObject, JsString};

pub fn new_string<'a>(data: &str) -> Handle<'a, JsString> {
    JsString::new(data)
}

pub fn new_number<'a, T: Into<f64>>(data: T) -> Handle<'a, JsNumber> {
    JsNumber::new(data)
}

pub fn empty_array<'a>() -> Handle<'a, JsArray> {
    JsArray::empty_array()
}

pub fn empty_object<'a>() -> Handle<'a, JsObject> {
    JsObject::empty_object()
}

pub fn new_array_buffer<'a>(data: &[u8]) -> Handle<'a, JsArrayBuffer> {
    JsArrayBuffer::new(data)
}
