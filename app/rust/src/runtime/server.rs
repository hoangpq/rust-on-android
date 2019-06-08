#![deny(rust_2018_idioms)]

use crate::runtime::OpAsyncFuture;
use std::env;
use std::net::SocketAddr;
use std::os::raw::c_char;
use tokio;
use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::*;

use crate::runtime::string_to_ptr;

extern "C" {
    fn lookup_deno_and_eval_script(uuid: u32, script: *const c_char);
}

pub fn create_server(uuid: u32) -> Result<OpAsyncFuture, Box<dyn std::error::Error>> {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>()?;

    let socket = TcpListener::bind(&addr)?;
    adb_debug!(format!("Listening on: {}", addr));

    let server = socket
        .incoming()
        .map_err(|e| adb_debug!(format!("failed to accept socket; error = {:?}", e)))
        .for_each(move |socket| {
            let (reader, writer) = socket.split();
            let amt = io::copy(reader, writer);

            let msg = amt.then(move |result| {
                match result {
                    Ok((amt, _, _)) => {}
                    Err(e) => adb_debug!(format!("error: {}", e)),
                }

                unsafe {
                    lookup_deno_and_eval_script(
                        uuid,
                        string_to_ptr("console.log(`Random: ${Math.random()}`)"),
                    );
                }

                Ok(())
            });

            tokio::spawn(msg)
        })
        .and_then(|_| future::ok(vec![0u8].into_boxed_slice()));

    Ok(Box::new(server))
}
