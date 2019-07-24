use std::marker::PhantomData;
use std::os::raw::c_void;

extern "C" {
    fn mem_same_handle(h1: Local, h2: Local) -> bool;
    fn new_primitive_number(local: &mut Local, v: f64);
    fn new_array(local: &mut Local, len: u32);
    fn new_array_buffer(local: &mut Local, data: *mut libc::c_void, byte_length: libc::size_t);
}

pub trait Value {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Local {
    pub handle: *mut c_void,
}

pub trait Managed: Copy {
    fn to_raw(self) -> Local;
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Handle<'a, T: Managed + 'a> {
    value: T,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: Managed + 'a> PartialEq for Handle<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { mem_same_handle(self.to_raw(), other.to_raw()) }
    }
}

impl<'a, T: Managed + 'a> Eq for Handle<'a, T> {}

impl<'a, T: Managed + 'a> Handle<'a, T> {
    pub fn to_raw(self) -> Local {
        return self.value.to_raw();
    }
    pub(crate) fn new_internal(value: T) -> Handle<'a, T> {
        Handle {
            value,
            phantom: PhantomData,
        }
    }
}

pub trait Object: Value {
    fn set<'a, K, V>(&self, key: Handle<K>, value: Handle<'a, V>) -> Handle<'a, V>
    where
        K: Managed + Value,
        V: Managed + Value,
    {
        value
    }
}

/// A Javascript value
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsValue(Local);

impl Managed for JsValue {
    fn to_raw(self) -> Local {
        self.0
    }
}

/// A JavaScript number.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsNumber(Local);

impl JsNumber {
    pub fn new<'a, T: Into<f64>>(x: T) -> Handle<'a, JsNumber> {
        JsNumber::new_internal(x.into())
    }
    pub(crate) fn new_internal<'a>(v: f64) -> Handle<'a, JsNumber> {
        unsafe {
            let mut local: Local = std::mem::zeroed();
            new_primitive_number(&mut local, v);
            Handle::new_internal(JsNumber(local))
        }
    }
}

impl Value for JsNumber {}

impl Managed for JsNumber {
    fn to_raw(self) -> Local {
        self.0
    }
}

/// A JavaScript object.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsObject(Local);

impl Value for JsObject {}

impl Managed for JsObject {
    fn to_raw(self) -> Local {
        self.0
    }
}

impl Object for JsObject {}

/// A Javascript array
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsArray(Local);

impl JsArray {
    pub fn new<'a>(len: u32) -> Handle<'a, JsArray> {
        unsafe {
            let mut local: Local = std::mem::zeroed();
            new_array(&mut local, len);
            Handle::new_internal(JsArray(local))
        }
    }
    pub fn empty_array<'a>() -> Handle<'a, JsArray> {
        JsArray::new(0)
    }
}

impl Value for JsArray {}
impl Object for JsArray {}

impl Managed for JsArray {
    fn to_raw(self) -> Local {
        self.0
    }
}

/// A Javascript arraybuffer
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsArrayBuffer(Local);

#[allow(non_snake_case)]
impl JsArrayBuffer {
    pub fn new<'a>(data: &[u8]) -> Handle<'a, JsArrayBuffer> {
        unsafe {
            let ptr = data.as_ptr() as *mut libc::c_void;
            let mut local: Local = std::mem::zeroed();
            new_array_buffer(&mut local, ptr, data.len());
            let _ = std::slice::from_raw_parts(ptr, data.len());
            Handle::new_internal(JsArrayBuffer(local))
        }
    }
}

impl Value for JsArrayBuffer {}

impl Managed for JsArrayBuffer {
    fn to_raw(self) -> Local {
        self.0
    }
}

/// A Javascript function
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsFunction<T: Object = JsObject> {
    raw: Local,
    marker: PhantomData<T>,
}

impl Value for JsFunction {}

impl Managed for JsFunction {
    fn to_raw(self) -> Local {
        self.raw
    }
}

impl<CL: Object> JsFunction<CL> {
    pub fn call<'a, T: 'a, A>(self, args: A) -> Handle<'a, JsNumber>
    where
        T: Managed + Value,
        A: IntoIterator<Item = Handle<'a, T>>,
    {
        // let mut args = args.into_iter().collect::<Vec<_>>();
        JsNumber::new(103)
    }
}
