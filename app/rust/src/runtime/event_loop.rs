use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::thread;

use futures::{Future, TryFutureExt};
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

use crate::runtime::isolate;
use crate::runtime::isolate::Isolate;

#[no_mangle]
pub extern "C" fn init_event_loop() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let (shutdown_send, mut shutdown_recv) = mpsc::unbounded_channel::<i32>();

    rt.block_on(async move {
        let mut isolate = Isolate::new();

        isolate.execute(
            r#"
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
                  '#7f8c8d',
                ];
                
                function format(value) {
                  return value > 9 ? value : '0' + value;
                }
                
                function createTimeString() {
                  const date = new Date();
                  const h = date.getHours();
                  const m = date.getMinutes();
                  const s = date.getSeconds();
                  return `${format(h)} : ${format(m)} : ${format(s)}`;
                }
                
                // color
                setInterval(() => {
                  const color = colorList[Math.ceil(Math.random() * colorList.length)];
                  context.setTextColor(color);
                }, 2000);
                
                // clock
                setInterval(() => {
                  context.setText(createTimeString());
                }, 1000);
                
                // Send to Rust world by ArrayBuffer
                const ab = new ArrayBuffer(10);
                const bufView = new Uint8Array(ab);
                
                $sendBuffer(ab, (data) => {
                  return {
                    ...data,
                    getName() {
                      return this.name;
                    },
                    getPromise() {
                      return Promise.resolve('hoangpq');
                    },
                  };
                });
                
                const locations = ['san', 'london'];
                
                function fetchWeatherInfo(location) {
                  return fetch(`https://www.metaweather.com/api/location/search/?query=${location}`)
                    .then((resp) => resp.json());
                }
                
                Promise.all(locations.map(fetchWeatherInfo))
                    .then((data) => {
                        console.log(JSON.stringify(data));
                    })
                    .catch((e) => console.log(e.message));
                
            "#,
        );

        isolate.await;
        shutdown_recv.recv().await
    });

    adb_debug!("Tear down Isolate!!");
}
