extern crate libc;

use libc::size_t;
use std::os::raw::c_void;

extern "C" {
    fn v8_function_cast(val: &Value) -> Function;
    fn v8_function_call(this: &Function, argc: i32, argv: *mut c_void);
    fn v8_buffer_new(data: *mut c_void, byte_length: size_t) -> ArrayBuffer;
}

pub trait ValueT {
    fn as_val(&self) -> &Value;
}

#[repr(C)]
pub struct Value(*mut *mut Value);

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
pub struct Context(*mut Context);

impl ValueT for &Context {
    fn as_val(&self) -> &Value {
        unimplemented!()
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Function(*mut *mut Function);

#[allow(non_snake_case)]
impl Function {
    pub fn Cast(val: &Value) -> Function {
        unsafe { v8_function_cast(val) }
    }
    pub fn Call<T>(&self, argc: i32, argv: &mut Vec<T>) {
        unsafe { v8_function_call(self, argc, argv.as_mut_ptr() as *mut c_void) }
    }
}
