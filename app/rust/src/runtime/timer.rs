use std::os::raw::c_void;
use std::pin::Pin;
use std::time::{Duration, Instant};

use futures::FutureExt;
use futures::{future, TryFutureExt};
use num::one;

use crate::runtime::isolate::Isolate;
use crate::runtime::DenoC;

extern "C" {
    fn resolve(d: *const DenoC, promise_id: u32, data: *const libc::c_char);
    fn invoke_timer_callback(d: *const DenoC);
    fn get_min_timeout(d: *const DenoC) -> u32;
}

pub fn invoke_timer_cb(isolate: &mut Isolate) {
    unsafe {
        let deno_ref = isolate.deno.as_ref();
        invoke_timer_callback(deno_ref.unwrap());
    }
}

pub fn min_timeout(isolate: &mut Isolate) -> u32 {
    unsafe {
        let deno_ref = isolate.deno.as_ref();
        get_min_timeout(deno_ref.unwrap())
    }
}

#[no_mangle]
pub fn timer(isolate_ptr: *const c_void, timeout: u32, promise_id: u32) {
    let isolate = unsafe { Isolate::from_raw_ptr(isolate_ptr) };
}
