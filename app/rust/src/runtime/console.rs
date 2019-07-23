use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use crate::v8::CallbackInfo;

lazy_static! {
    static ref TIME_TABLE: Mutex<HashMap<&'static str, Instant>> = Mutex::new(HashMap::new());
}

#[no_mangle]
pub extern "C" fn console_time(args: &CallbackInfo) {
    let raw = args.Get(0).to_string();
    let s = rust_str!(raw);
    let mut table = TIME_TABLE.lock().unwrap();
    table.insert(s, Instant::now());
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn console_time_end(args: &CallbackInfo) {
    let raw = args.Get(0).to_string();
    let s = rust_str!(raw);
    let mut table = TIME_TABLE.lock().unwrap();
    if let Some(instant) = table.get_mut(&s) {
        adb_debug!(format!("{}: {}ms", s, instant.elapsed().as_millis()));
    }
}
