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
                setInterval(() => {}, 100);
                
                const Random = java.import('java/util/Random');
                const random = new Random(10000);
                 
                console.log(`nextInt: ${random.nextInt()}`);
                console.log(`nextDouble: ${random.nextDouble()}`);
                
                const context = java.import('context');
                const colorList = [
                    '#FFBF00',
                    '#9966CC',
                    '#FBCEB1',
                    '#7FFFD4',
                    '#007FFF',
                    '#89CFF0',
                    '#F5F5DC',
                    '#0000FF',
                    '#0095B6',
                    '#8A2BE2',
                    '#DE5D83',
                    '#CD7F32',
                    '#964B00',
                    '#800020'
                ];
                
                function changeColor(context) {
                    setInterval(async () => {
                        const color = colorList[Math.ceil(Math.random() * colorList.length)];
                        const data = await context.setBackgroundColor(color);
                        console.log(data);
                    }, 1000);
                }
                
                changeColor(context);
                
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
