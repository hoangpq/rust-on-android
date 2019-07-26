use crate::sys::types::{Handle, Local, Value};

pub type FunctionCallback = unsafe extern "C" fn(args: &CallbackInfo);
pub type FunctionCallbackInfo = libc::c_void;

extern "C" {
    fn set_return_value(info: &FunctionCallbackInfo, value: Local);
}

#[repr(C)]
pub struct CallbackInfo {
    info: FunctionCallbackInfo,
}

impl CallbackInfo {
    pub fn set_return_value<T: Value>(&self, value: Handle<T>) {
        unsafe { set_return_value(&self.info, value.to_raw()) }
    }
}
