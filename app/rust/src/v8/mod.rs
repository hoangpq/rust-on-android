extern crate libc;

use libc::size_t;
use std::os::raw::c_void;

extern "C" {
    fn v8_function_cast(v: Value) -> Function;
    fn v8_function_call(f: Function, argc: i32, argv: *mut c_void);
    fn v8_buffer_new(data: *mut c_void, byte_length: size_t) -> ArrayBuffer;
    fn v8_function_callback_info_get(info: &FunctionCallbackInfo, index: i32) -> Value;
    fn v8_function_callback_length(info: &FunctionCallbackInfo) -> i32;
}

#[repr(C)]
#[derive(Debug)]
pub struct Value(*mut Value);

pub trait ValueT {
    fn as_val(&self) -> &Value;
}

#[repr(C)]
#[derive(Debug)]
pub struct ArrayBuffer(*mut ArrayBuffer);

#[allow(non_snake_case)]
impl ArrayBuffer {
    pub fn New(data: &[u8]) -> ArrayBuffer {
        let ptr = data.as_ptr() as *mut c_void;
        unsafe { v8_buffer_new(ptr, data.len()) }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Context(*mut Context);

#[repr(C)]
#[derive(Debug)]
pub struct Function(*mut Function);

#[allow(non_snake_case)]
impl Function {
    pub fn Cast(val: Value) -> Function {
        unsafe { v8_function_cast(val) }
    }
    pub fn Call<T>(self, mut args: Vec<T>) {
        let argv = args.as_mut_ptr();
        let argc = args.len() as i32;
        unsafe { v8_function_call(self, argc, argv as *mut c_void) }
    }
}

pub type FunctionCallbackInfo = c_void;

#[derive(Debug)]
#[repr(C)]
pub struct CallbackInfo {
    info: FunctionCallbackInfo,
}

#[allow(non_snake_case)]
impl CallbackInfo {
    pub fn Get(&self, i: i32) -> Value {
        unsafe {
            v8_function_callback_info_get(&self.info, i)
        }
    }
    pub fn Len(&self) -> i32 {
        unsafe { v8_function_callback_length(&self.info) }
    }
}