use jni::JNIEnv;

use crate::runtime::isolate::Isolate;
use crate::runtime::{create_thread_pool_runtime, ptr_to_string};
use std::os::raw::c_char;

#[no_mangle]
pub unsafe extern "C" fn adb_debug(p: *mut c_char) {
    if let Some(msg) = ptr_to_string(p) {
        adb_debug!(msg);
    }
}

#[no_mangle]
pub unsafe fn init_event_loop(_env: &'static JNIEnv) {
    let main_future = futures::lazy(move || {
        let isolate = Isolate::new();
        isolate.execute(
            r#"
                const users = ['hoangpq', 'firebase'];

                function fetchUserInfo(user) {
                    return fetch(`https://api.github.com/users/${user}`);
                }

                setInterval(() => {
                    $log('interval 5s');
                }, 5000);

                setInterval(() => {
                    $log('interval 4s');
                }, 4000);

                console.time('timer2');
                setTimeout(() => {
                    console.timeEnd('timer2');
                }, 5000);

                console.time('api call');
                Promise.all(users.map(fetchUserInfo))
                    .then(data => {
                        $log(`Name: ${data}`);
                        console.timeEnd('api call');
                    });
            "#,
        );
        isolate
    });

    let rt = create_thread_pool_runtime();
    rt.block_on_all(main_future).unwrap();
}
