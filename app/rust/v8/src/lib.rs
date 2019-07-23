extern crate libc;

use libc::size_t;
use std::ffi::CString;
use std::mem;
use std::os::raw::{c_char, c_void};

#[allow(non_camel_case_types)]
#[allow(dead_code)]
pub type FunctionCallback = unsafe extern "C" fn(args: &CallbackInfo);

extern "C" {
    fn v8_function_cast(v: Value) -> Function;
    fn v8_buffer_new(data: *mut c_void, byte_length: size_t) -> ArrayBuffer;
    fn v8_function_call(f: Function, argc: i32, argv: *mut c_void);
    fn v8_function_callback_info_get(info: &FunctionCallbackInfo, index: i32) -> Value;
    fn v8_function_callback_length(info: &FunctionCallbackInfo) -> i32;
    fn v8_set_return_value(info: &FunctionCallbackInfo, val: &Value);
    fn v8_string_new_from_utf8(data: *const c_char) -> String;
    fn v8_value_into_raw(value: Value) -> *mut c_char;
    fn v8_number_from_raw(number: u64) -> Number;
}

pub trait ValueT {
    fn as_val(&self) -> &Value;
}

macro_rules! value_method (
    ($ty:ident) => {
        impl ValueT for $ty {
            fn as_val(&self) -> &Value {
                unsafe { mem::transmute(self) }
            }
        }
    }
);

#[repr(C)]
#[derive(Debug)]
pub struct Value(*mut Value);
value_method!(Value);

impl Value {
    pub fn to_string(self) -> *mut c_char {
        unsafe { v8_value_into_raw(self) }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Context(*mut Context);

#[repr(C)]
#[derive(Debug)]
pub struct ArrayBuffer(*mut ArrayBuffer);
value_method!(ArrayBuffer);

#[allow(non_snake_case)]
impl ArrayBuffer {
    pub fn New(data: &[u8]) -> ArrayBuffer {
        let ptr = data.as_ptr() as *mut c_void;
        unsafe { v8_buffer_new(ptr, data.len()) }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Function(*mut Function);
value_method!(Function);

unsafe impl Send for Function {}

unsafe impl Sync for Function {}

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
        unsafe { v8_function_callback_info_get(&self.info, i) }
    }
    pub fn Len(&self) -> i32 {
        unsafe { v8_function_callback_length(&self.info) }
    }
    pub fn SetReturnValue<T: ValueT>(&self, v: T) {
        unsafe { v8_set_return_value(&self.info, v.as_val()) }
    }
}

#[repr(C)]
pub struct String(*mut String);
value_method!(String);

#[allow(non_snake_case)]
impl String {
    pub fn NewFromUtf8(data: &str) -> String {
        let data = CString::new(data).unwrap();
        unsafe { v8_string_new_from_utf8(data.as_ptr()) }
    }
}

#[repr(C)]
pub struct Number(*mut Number);
value_method!(Number);

#[allow(non_snake_case)]
impl Number {
    pub fn new(number: u64) -> Self {
        unsafe { v8_number_from_raw(number) }
    }
}
