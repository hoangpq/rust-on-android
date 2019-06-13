use std::convert::Into;
use std::time::{Duration, Instant};

use futures::Future;
use tokio_timer::Delay;

use crate::runtime::stream_cancel::TimerCancel;

pub fn panic_on_error<I, E, F>(f: F) -> impl Future<Item = I, Error = ()>
where
    F: Future<Item = I, Error = E>,
    E: std::fmt::Debug,
{
    f.map_err(|err| adb_debug!(format!("Future got unexpected error: {:?}", err)))
}

pub fn set_timeout(delay: u32) -> (impl Future<Item = (), Error = ()>, TimerCancel) {
    let (tx, rx) = futures::sync::oneshot::channel::<()>();
    let duration = Duration::from_millis(delay.into());

    let delay_task = Delay::new(Instant::now() + duration)
        .map_err(|err| adb_debug!(format!("Future got unexpected error: {:?}", err)))
        // .select(rx.map_err(|_| ()))
        .then(|_| Ok(()));

    (delay_task, TimerCancel(Some(tx)))
}
