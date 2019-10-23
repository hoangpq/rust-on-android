use std::sync::{Arc, Mutex};
use std::thread;

use futures::{Async, Future};

use crate::runtime::isolate;

#[derive(Clone)]
pub struct Worker {
    inner: Arc<Mutex<isolate::Isolate>>,
}

impl Worker {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(isolate::Isolate::new())),
        }
    }

    fn execute(&mut self, script: &str) {
        let mut isolate = self.inner.lock().unwrap();
        isolate.execute(script);
    }
}

impl Future for Worker {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let mut isolate = self.inner.lock().unwrap();
        isolate.poll().map_err(|err| adb_debug!(err))
    }
}

#[no_mangle]
pub extern "C" fn init_event_loop() {
    thread::spawn(move || {
        let main_future = futures::lazy(move || {
            let mut worker = Worker::new();
            worker.execute(
                r#"
                // to keep event loop alive
                setInterval(() => {
                    console.log('interval 500');
                }, 500);
                
                const Random = java.import('java/util/Random');
                const random = new Random(10000);
                 
                console.log(`nextInt: ${random.nextInt()}`);
                console.log(`nextDouble: ${random.nextDouble()}`);
                
                const context = java.import('context');
                const colorList = [
                    '#1abc9c',
                    '#2ecc71',
                    '#3498db',
                    '#9b59b6',
                    '#34495e',
                    '#16a085',
                    '#27ae60',
                    '#2980b9',
                    '#8e44ad',
                    '#2c3e50',
                    '#f1c40f',
                    '#e67e22',
                    '#e74c3c',
                    '#ecf0f1',
                    '#95a5a6',
                    '#f39c12',
                    '#d35400',
                    '#c0392b',
                    '#bdc3c7',
                    '#7f8c8d'
                ];
                
                function changeColor(context) {
                    setInterval(async () => {
                        const color = colorList[Math.ceil(Math.random() * colorList.length)];
                        const colorCode = await context.setBackgroundColor(color);
                        console.log(colorCode);
                    }, 2000);
                }
                
                function format(value) {
                    return value > 9 ? value: '0' + value;
                }
                
                function createTimeString() {
                    const date = new Date();
                    const h = date.getHours();
                    const m = date.getMinutes() + 1;
                    const s = date.getSeconds();
                    return `${format(h)}:${format(m)}:${format(s)}`;
                }
                
                function clock(context) {
                    setInterval(async () => {
                        await context.setText(createTimeString());
                    }, 500);
                }
                
                changeColor(context);
                clock(context);
                
                // Send to Rust world by ArrayBuffer
                const ab = new ArrayBuffer(10);
                const bufView = new Uint8Array(ab);

                $sendBuffer(ab, data => {
                    return {
                        ...data,
                        getName() { return this.name; },
                        getPromise() {
                            /*return fetch('https://api.github.com/users/ardanlabs')
                                .then(resp => resp.json())
                                .then(user => user.name);*/
                            return Promise.resolve('hoangpq');
                        },
                    };
                });

                const users = ['hoangpq', 'firebase'];

                function fetchUserInfo(user) {
                    return fetch(`https://api.github.com/users/${user}`)
                        .then(resp => resp.json());
                }

                const start = Date.now();
                setTimeout(() => {
                    console.log(`timeout 3s: ${Date.now() - start}`);
                }, 3000);

                /*Promise.all(users.map(fetchUserInfo))
                    .then(data => {
                        const names = data.map(user => user.name).join(', ');
                        console.log(`Name: ${names}`);
                        console.log(`api call: ${Date.now() - start}`);
                    })
                    .catch(e => console.log(e.message));*/
            "#,
            );
            worker
        });

        tokio::runtime::current_thread::run(main_future);
    });
}
