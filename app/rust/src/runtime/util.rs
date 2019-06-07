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
pub unsafe extern "C" fn init_event_loop(_env: &'static JNIEnv) {
    let main_future = futures::lazy(move || {
        let isolate = Isolate::new();
        isolate.execute(
            r#"
                const users = ['hoangpq', 'firebase'];

                function fetchUserInfo(user) {
                    return fetch(`https://api.github.com/users/${user}`)
                        .then(resp => resp.json());
                }

                const t1 = setInterval(() => {
                    console.log('interval 500ms');
                }, 500);

                const start = Date.now();
                setTimeout(() => {
                    console.log(`timeout 5s: ${Date.now() - start}`);
                    clearInterval(t1);
                }, 5000);

                Promise.all(users.map(fetchUserInfo))
                    .then(data => {
                        const names = data.map(user => user.name).join(', ');
                        console.log(`Name: ${names}`);
                        console.log(`api call: ${Date.now() - start}`);
                    });

                // fetch json api
                fetch('https://freejsonapi.com/posts')
                    .then(resp => resp.json())
                    .then(resp => {
                        console.log(resp.data.length);
                    });

            "#,
        );
        isolate
    });

    let rt = create_thread_pool_runtime();
    rt.block_on_all(main_future).unwrap();
    adb_debug!("Done");
}
