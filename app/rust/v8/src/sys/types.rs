use crate::sys::util::to_c_str;
use std::borrow::Cow;
use std::fmt::{Debug, Error, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::os::raw::{c_char, c_void};

extern "C" {
    fn mem_same_handle(h1: Local, h2: Local) -> bool;
    fn new_primitive_number(local: &mut Local, v: f64);
    fn new_object(local: &mut Local);
    fn new_array(local: &mut Local, len: u32);
    fn new_array_buffer(local: &mut Local, data: *mut libc::c_void, byte_length: libc::size_t);
    fn new_utf8_string(local: &mut Local, data: *const u8, len: u32);
    fn function_call(
        out: &mut Local,
        local: Local,
        this: Local,
        argc: u32,
        argv: *mut c_void,
    ) -> bool;
    fn raw_value(val: Local) -> *const c_char;
    fn object_set(out: &mut bool, obj: Local, key: Local, value: Local) -> bool;
    fn object_index_set(out: &mut bool, obj: Local, index: u32, value: Local) -> bool;
    fn object_string_set(
        out: &mut bool,
        obj: Local,
        ptr: *const u8,
        len: u32,
        value: Local,
    ) -> bool;
    fn object_string_get(out: &mut Local, obj: Local, ptr: *const u8, len: u32) -> bool;
    fn primitive_null(out: &mut Local);
}

pub struct Utf8<'a> {
    contents: Cow<'a, str>,
}

impl<'a> From<&'a str> for Utf8<'a> {
    fn from(s: &'a str) -> Self {
        Utf8 {
            contents: Cow::from(s),
        }
    }
}

impl<'a> Utf8<'a> {
    pub fn lower(&self) -> (*const u8, u32) {
        (self.contents.as_ptr(), self.contents.len() as u32)
    }
}

pub trait Managed: Copy {
    fn to_raw(self) -> Local;

    fn from_raw(h: Local) -> Self;
}

fn build(s: &str) -> (*const u8, u32) {
    unsafe {
        let ptr = to_c_str(s);
        let len = libc::strlen(ptr);
        (ptr as *const u8, len as u32)
    }
}

/// A property key in Javascript object
pub trait PropertyKey {
    unsafe fn get_from(self, out: &mut Local, obj: Local) -> bool;
    unsafe fn set_from(self, out: &mut bool, obj: Local, val: Local) -> bool;
}

impl PropertyKey for u32 {
    unsafe fn get_from(self, out: &mut Local, obj: Local) -> bool {
        unimplemented!()
    }

    unsafe fn set_from(self, out: &mut bool, obj: Local, val: Local) -> bool {
        object_index_set(out, obj, self, val)
    }
}

impl<'a, K: Value> PropertyKey for Handle<'a, K> {
    unsafe fn get_from(self, out: &mut Local, obj: Local) -> bool {
        unimplemented!()
    }

    unsafe fn set_from(self, out: &mut bool, obj: Local, val: Local) -> bool {
        object_set(out, obj, self.to_raw(), val)
    }
}

impl<'a> PropertyKey for &'a str {
    unsafe fn get_from(self, out: &mut Local, obj: Local) -> bool {
        let (ptr, len) = build(self);
        object_string_get(out, obj, ptr, len)
    }

    unsafe fn set_from(self, out: &mut bool, obj: Local, val: Local) -> bool {
        let (ptr, len) = build(self);
        object_string_set(out, obj, ptr, len, val)
    }
}

/// The trait shared by all JavaScript values.
pub trait Value: Managed {}

/// &str into Handle<JsString>
impl<'a> Into<Handle<'a, JsString>> for &str {
    fn into(self) -> Handle<'a, JsString> {
        JsString::new(self)
    }
}

/// i32 into Handle<JsNumber>
impl<'a> Into<Handle<'a, JsNumber>> for i32 {
    fn into(self) -> Handle<'a, JsNumber> {
        JsNumber::new(self)
    }
}

/// Vector into Handle<JsArray>
impl<'a> Into<Handle<'a, JsArray>> for Vec<&str> {
    fn into(self) -> Handle<'a, JsArray> {
        let array = JsArray::empty_array();
        for (i, e) in self.iter().enumerate() {
            array.set_from_raw(i as u32, *e);
        }
        array
    }
}

pub trait Object: Value {
    fn set<'a, K: PropertyKey, V: Value>(&self, key: K, val: Handle<'a, V>) {
        unsafe {
            let mut result = false;
            key.set_from(&mut result, self.to_raw(), val.to_raw());
        };
    }
    fn get<'a, T: Value, K: PropertyKey>(&self, key: K) -> Handle<'a, T> {
        unsafe {
            let mut out: Local = std::mem::zeroed();
            key.get_from(&mut out, self.to_raw());
            Handle::new_internal(T::from_raw(out))
        }
    }
    fn set_from_raw<'a, T: 'a, K: PropertyKey, V: Into<Handle<'a, T>>>(&self, key: K, val: V)
    where
        T: Value,
    {
        unsafe {
            let mut result = false;
            key.set_from(&mut result, self.to_raw(), val.into().to_raw());
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Local {
    pub handle: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Handle<'a, T: Managed + 'a> {
    value: T,
    phantom: PhantomData<&'a T>,
}

impl<'a, T: Managed> Debug for Handle<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", rust_str!(raw_value(self.to_raw())))
    }
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

impl<'a, T: Managed> Deref for Handle<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<'a, T: Managed> DerefMut for Handle<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

/// A Javascript value.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsValue(Local);

impl Managed for JsValue {
    fn to_raw(self) -> Local {
        self.0
    }

    fn from_raw(h: Local) -> Self {
        JsValue(h)
    }
}

impl Value for JsValue {}
impl Object for JsValue {}

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

    fn from_raw(h: Local) -> Self {
        JsNumber(h)
    }
}

/// A JavaScript object.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsObject(Local);

impl JsObject {
    pub fn empty_object<'a>() -> Handle<'a, JsObject> {
        JsObject::new_internal()
    }

    pub(crate) fn new_internal<'a>() -> Handle<'a, JsObject> {
        unsafe {
            let mut local: Local = std::mem::zeroed();
            new_object(&mut local);
            Handle::new_internal(JsObject(local))
        }
    }
}

impl Managed for JsObject {
    fn to_raw(self) -> Local {
        self.0
    }

    fn from_raw(h: Local) -> Self {
        JsObject(h)
    }
}

impl Value for JsObject {}
impl Object for JsObject {}

/// A Javascript array.
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

    fn from_raw(h: Local) -> Self {
        JsArray(h)
    }
}

/// A Javascript string.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsString(Local);

impl JsString {
    pub fn new<'a>(data: &str) -> Handle<'a, JsString> {
        JsString::new_internal(data)
    }

    pub(crate) fn new_internal<'a>(data: &str) -> Handle<'a, JsString> {
        unsafe {
            let (ptr, len) = Utf8::from(data).lower();
            let mut local: Local = std::mem::zeroed();
            new_utf8_string(&mut local, ptr, len);
            Handle::new_internal(JsString(local))
        }
    }
}

impl Value for JsString {}
impl Object for JsString {}

impl Managed for JsString {
    fn to_raw(self) -> Local {
        self.0
    }

    fn from_raw(h: Local) -> Self {
        JsString(h)
    }
}

/// A Javascript arraybuffer.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsArrayBuffer(Local);

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

    fn from_raw(h: Local) -> Self {
        JsArrayBuffer(h)
    }
}

/// A Javascript function.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsFunction<T: Object = JsObject> {
    raw: Local,
    marker: PhantomData<T>,
}

impl<CL: Object> JsFunction<CL> {
    pub fn call<'a, 'b, T, R, A, AS>(self, this: Handle<'a, T>, args: AS) -> Handle<'a, R>
    where
        T: Value + 'a,
        A: Value + 'b,
        R: Value + 'b,
        AS: IntoIterator<Item = Handle<'b, A>>,
    {
        let mut args = args.into_iter().collect::<Vec<_>>();
        unsafe {
            let mut local: Local = std::mem::zeroed();
            function_call(
                &mut local,
                self.to_raw(),
                this.to_raw(),
                args.len() as u32,
                args.as_mut_ptr() as *mut c_void,
            );
            Handle::new_internal(R::from_raw(local))
        }
    }
}

impl<T: Object> Object for JsFunction<T> {}
impl<T: Object> Value for JsFunction<T> {}

impl<T: Object> Managed for JsFunction<T> {
    fn to_raw(self) -> Local {
        self.raw
    }

    fn from_raw(h: Local) -> Self {
        JsFunction {
            raw: h,
            marker: PhantomData,
        }
    }
}

/// A Javascript null.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct JsNull(Local);

impl JsNull {
    pub fn new<'a>() -> Handle<'a, JsNull> {
        JsNull::new_internal()
    }

    pub(crate) fn new_internal<'a>() -> Handle<'a, JsNull> {
        unsafe {
            let mut local: Local = std::mem::zeroed();
            primitive_null(&mut local);
            Handle::new_internal(JsNull(local))
        }
    }
}

impl Value for JsNull {}
impl Object for JsNull {}

impl Managed for JsNull {
    fn to_raw(self) -> Local {
        self.0
    }

    fn from_raw(h: Local) -> Self {
        JsNull(h)
    }
}
