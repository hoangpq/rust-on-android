use libc;
use std::borrow::Cow;

extern "C" {
    fn as_char_ptr(data: *const u8) -> *const libc::c_char;
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
    pub fn as_char_ptr(&self) -> *const libc::c_char {
        unsafe { as_char_ptr(self.contents.as_ptr()) }
    }
}
