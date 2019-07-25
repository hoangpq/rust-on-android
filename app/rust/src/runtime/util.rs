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
                /** Text decoder */
                function TextDecoder() {}

                TextDecoder.prototype.decode = function(octets) {
                    var string = '';
                    var i = 0;
                    while (i < octets.length) {
                        var octet = octets[i];
                        var bytesNeeded = 0;
                        var codePoint = 0;
                        if (octet <= 0x7f) {
                            bytesNeeded = 0;
                            codePoint = octet & 0xff;
                        } else if (octet <= 0xdf) {
                            bytesNeeded = 1;
                            codePoint = octet & 0x1f;
                        } else if (octet <= 0xef) {
                            bytesNeeded = 2;
                            codePoint = octet & 0x0f;
                        } else if (octet <= 0xf4) {
                            bytesNeeded = 3;
                            codePoint = octet & 0x07;
                        }
                        if (octets.length - i - bytesNeeded > 0) {
                            var k = 0;
                            while (k < bytesNeeded) {
                                octet = octets[i + k + 1];
                                codePoint = (codePoint << 6) | (octet & 0x3f);
                                k += 1;
                            }
                        } else {
                            codePoint = 0xfffd;
                            bytesNeeded = octets.length - i;
                        }
                        string += String.fromCodePoint(codePoint);
                        i += bytesNeeded + 1;
                    }
                    return string;
                };

                // Send to Rust world by ArrayBuffer
                const ab = new ArrayBuffer(10);
                const bufView = new Uint8Array(ab);

                $sendBuffer(ab, function(buf) {
                  const ar = new Uint8Array(buf);
                  console.log(`-> Received ${new TextDecoder().decode(ar)} from Rust <-`);
                  return ar.length;
                });

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
