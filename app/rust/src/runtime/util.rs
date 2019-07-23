use std::thread;

use libc::c_char;

use crate::runtime::{create_thread_pool_runtime, Worker};

#[no_mangle]
pub extern "C" fn adb_debug(p: *mut c_char) {
    adb_debug!(rust_str!(p));
}

#[no_mangle]
pub extern "C" fn init_event_loop() {
    thread::spawn(move || {
        let main_future = futures::lazy(move || {
            let mut worker = Worker::new();
            worker.execute(
                r#"

                try {
                    const val = $testFn(function() {});
                    console.log(val);
                } catch (e) {
                    console.log(e);
                }

                const users = ['hoangpq', 'firebase'];

                function fetchUserInfo(user) {
                    return fetch(`https://api.github.com/users/${user}`)
                        .then(resp => resp.json());
                }

                const timer02 = setInterval(() => {
                    console.log(`10s interval`);
                }, 10000);

                const start = Date.now();
                setTimeout(() => {
                    console.log(`timeout 3s: ${Date.now() - start}`);
                }, 3000);

                Promise.all(users.map(fetchUserInfo))
                    .then(data => {
                        const names = data.map(user => user.name).join(', ');
                        console.log(`Name: ${names}`);
                        console.log(`api call: ${Date.now() - start}`);
                    })
                    .catch(e => console.log(e.message));

                (async function() {
                    try {
                        // fetch json api
                        let resp = await fetch('https://freejsonapi.com/posts').then(resp => resp.json());
                        console.log(`Total: ${resp.data.length}`);
                    } catch (e) {
                        console.log(e.message);
                    }
                })();
            "#,
            );
            worker
        });

        tokio::runtime::current_thread::run(main_future);
    });
}
