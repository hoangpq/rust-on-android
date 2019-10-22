use crate::sys::types::{Handle, Local, Value};
use libc::c_void;

pub type FunctionCallback = extern "C" fn(args: &CallbackInfo);
pub type FunctionCallbackInfo = c_void;

extern "C" {
    fn set_return_value(info: &FunctionCallbackInfo, value: Local);
    fn callback_info_get(info: &FunctionCallbackInfo, index: u32, local: &mut Local);
}

#[repr(C)]
pub struct CallbackInfo {
    info: FunctionCallbackInfo,
}

impl CallbackInfo {
    pub fn set_return_value<T: Value>(&self, value: Handle<T>) {
        unsafe { set_return_value(&self.info, value.to_raw()) }
    }
    pub fn set_return_value_checked<T: Value>(&self, value: Handle<T>, cond: bool) {
        if cond {
            self.set_return_value(value);
        }
    }
    pub fn get<'a, T: Value>(&self, index: u32) -> Handle<'a, T> {
        unsafe {
            let mut local: Local = std::mem::zeroed();
            callback_info_get(&self.info, index, &mut local);
            Handle::new_internal(T::from_raw(local))
        }
    }
}
