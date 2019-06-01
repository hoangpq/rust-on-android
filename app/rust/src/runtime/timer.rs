use std::convert::Into;
use std::time::{Duration, Instant};

use futures::{future, Future};
use futures::stream::Stream;
use tokio_timer::{Delay, Interval};

use crate::runtime::stream_cancel::{StreamExt, TimerCancel};

pub fn panic_on_error<I, E, F>(f: F) -> impl Future<Item = I, Error = ()>
where
    F: Future<Item = I, Error = E>,
    E: std::fmt::Debug,
{
    f.map_err(|err| adb_debug!(format!("Future got unexpected error: {:?}", err)))
}

pub fn set_timeout<F>(cb: F, delay: u32) -> (impl Future<Item = (), Error = ()>, TimerCancel)
where
    F: FnOnce() -> (),
{
    let (tx, rx) = futures::sync::oneshot::channel::<()>();
    let duration = Duration::from_millis(delay.into());

    let delay_task = Delay::new(Instant::now() + duration)
        .map_err(|err| adb_debug!(format!("Future got unexpected error: {:?}", err)))
        .and_then(|_| {
            cb();
            Ok(())
        })
        .select(rx.map_err(|_| ()))
        .map(|_| ())
        .map_err(|_| ());

    (delay_task, TimerCancel(Some(tx)))
}

pub fn set_interval<F>(cb: F, delay: u32) -> (impl Future<Item = (), Error = ()>, TimerCancel)
where
    F: Fn() -> (),
{
    let delay = Duration::from_millis(delay.into());

    let (tx, rx) = futures::sync::oneshot::channel::<()>();
    let interval_task = Interval::new(Instant::now() + delay, delay)
        .take_until(rx.map_err(|_| ()))
        .for_each(move |_| {
            cb();
            future::ok(())
        })
        .map_err(|e| adb_debug!(e));

    (interval_task, TimerCancel(Some(tx)))
}
