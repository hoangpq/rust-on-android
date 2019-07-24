extern crate libc;

pub mod types;
use crate::types::{Handle, Local, Managed, Value};

#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub type FunctionCallback = unsafe extern "C" fn(args: &CallbackInfo);
pub type FunctionCallbackInfo = libc::c_void;

extern "C" {
    fn set_return_value(info: &FunctionCallbackInfo, value: Local);
}

#[repr(C)]
#[allow(non_snake_case)]
pub struct CallbackInfo {
    info: FunctionCallbackInfo,
}

impl CallbackInfo {
    pub fn set_return_value<T: Value + Managed>(&self, value: Handle<T>) {
        unsafe { set_return_value(&self.info, value.to_raw()) }
    }
}
