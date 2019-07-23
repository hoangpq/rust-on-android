use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use crate::runtime::ptr_to_string;
use crate::v8::CallbackInfo;

lazy_static! {
    static ref TIME_TABLE: Mutex<HashMap<String, Instant>> = Mutex::new(HashMap::new());
}

#[no_mangle]
pub unsafe extern "C" fn console_time(args: &CallbackInfo) {
    let raw = args.Get(0).to_string();
    if let Some(s) = ptr_to_string(raw) {
        let mut table = TIME_TABLE.lock().unwrap();
        table.insert(s, Instant::now());
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn console_time_end(args: &CallbackInfo) {
    let raw = args.Get(0).to_string();
    if let Some(s) = ptr_to_string(raw) {
        let mut table = TIME_TABLE.lock().unwrap();
        if let Some(instant) = table.get_mut(&s) {
            adb_debug!(format!("{}: {}ms", s, instant.elapsed().as_millis()));
        }
    }
}
