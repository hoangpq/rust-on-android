use jni::JNIEnv;

use crate::runtime::isolate::Isolate;
use crate::runtime::{create_thread_pool_runtime, ptr_to_string};

#[no_mangle]
pub unsafe extern "C" fn adb_debug(p: *mut libc::c_char) {
    if let Some(msg) = ptr_to_string(p) {
        adb_debug!(msg);
    }
}

#[no_mangle]
pub unsafe fn init_event_loop(_env: &'static JNIEnv) {
    let main_future = futures::lazy(move || {
        let mut isolate = Isolate::new();
        isolate.vexecute(
            r#"
               const data = { msg: 'Hello, World!' };

               const t1 = $timeout((msg) => { $log('$timeout 5s'); }, 5e3);
               const t2 = $timeout((msg) => { $log('$timeout 6s'); }, 6e3);
               const t3 = $timeout((msg) => { $log('$timeout 7s'); }, 7e3);

               // test to clear timeout
               $timeout((msg) => {
                  $clear(t1);
                  $clear(t3);
               }, 0);

               $log(data.msg);
            "#,
        );
        isolate
    });

    let mut rt = create_thread_pool_runtime();
    rt.block_on_all(main_future).unwrap();
}
